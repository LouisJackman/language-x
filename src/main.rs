mod lexing;

use std::env::{Args, args};
use std::fs::File;
use std::io::Read;

use lexing::{LexedToken, Lexer};
use lexing::source::Source;
use lexing::tokens::Token;

fn load_source(args: Args) -> String {
    let args_vector: Vec<String> = args.collect();
    if args_vector.len() <= 1 {
        panic!("source path arg missing");
    }
    let source_path = &args_vector[1];
    let mut file = File::open(source_path).expect("could not open specified source file");
    let mut source = String::new();
    file.read_to_string(&mut source).expect(
        "failed to read source file contents",
    );
    source
}

fn demo(lexer: &mut Lexer) {
    loop {
        match lexer.lex() {
            Ok(LexedToken { token: Token::Eof, .. }) => break,
            Ok(LexedToken { token, .. }) => println!("{:?}", token),
            Err(e) => panic!(e),
        }
    }
}

fn main() {
    let source = load_source(args());
    let mut lexer = Lexer::from(Source::from(source.chars().collect()));
    demo(&mut lexer)
}
