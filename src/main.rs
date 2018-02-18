mod lexing;
mod parsing;
mod peekable_buffer;

use std::env::{Args, args};
use std::fs::File;
use std::io::{Read};

use lexing::lexer::{LexedToken, Lexer};
use lexing::source::Source;
use lexing::tokens::Token;

fn load_source(args: Args) -> String {
    let args_vector: Vec<String> = args.collect();
    if args_vector.len() <= 1 {
        panic!("source path arg missing");
    }

    let source_path = &args_vector[1];

    let mut file = File
        ::open(source_path)
        .expect("could not open specified source file");

    let mut source = String::new();
    file.read_to_string(&mut source).expect(
        "failed to read source file contents",
    );
    source
}

fn demo(lexer: Lexer) {
    let (rx, join_handle) = lexer.lex().unwrap();
    loop {
        match rx.recv() {
            Ok(LexedToken { token: Token::Eof, .. }) => break,
            Ok(LexedToken { token, .. }) => println!("{:?}", token),
            Err(e) => panic!(e),
        }
    }
    join_handle.join().unwrap();
}

fn main() {
    let source_string = load_source(args());
    let source = Source::from(source_string.chars().collect::<Vec<char>>());
    let lexer = Lexer::from(source);
    demo(lexer);
}
