//! # The Sylan Programming Language
//!
//! These RustDoc comments here document the language's implementation. To see an overview of the
//! language itself, its raison detre, decision rationales, and broader non-technical architecture,
//! see the documentation at `docs/index.html`.
//!
//! ## Modules
//!
//! `main.rs` stitches the whole system together by building a dependency and execution order chain
//! between the modules:
//! ```
//!                                                        ,-> interpreter -> runtime
//! lexing -> parsing -> simplification -> IL -> backend -<
//!                                                        `-> runtime -> compiler
//! ```
//!
//! The interpreter invokes the runtime whereas the runtime is baked into the compiled artefact,
//! and is only actually run when the resulting executable is run.
//!
//! TODO: consider whether each of these modules should actually be a crate.
//!
//! ## Concurrency and Parallelism
//!
//! Note that "execution order" is a logical order not a literal execution order, as Sylan is
//! multithreaded. The threads look like this, some of which is already implemented and while
//! other parts just being the proposals so far:
//!
//! Lexer:
//! * The lexer thread.
//! * Emits tokens over a channel.
//!
//! Parser:
//! * The parser thread.
//! * An additional parsing excursion thread.
//! * Receives tokens from a channel.
//! * Each parser can create a single excursion, creating a new parser with its own thread.
//!   - That new parser itself can create a new excursion ad infinium, although this capability
//!     should not be utilised in order to keep the parser simple.
//!   - Therefore only one parsing excursion thread exists alongside the main parsing thread in
//!     practice.
//!   - Parsers relay received tokens over their channel to excursion child parsers, also via a
//!     channel for each.
//! * As the entire AST is built before moving on, this is not done in a dedicated thread.
//!   - TODO: Perhaps a lazy functional zipper data structure could be used by the AST to allow
//!     lazily building the AST in the background, allowing the parser to be in its own thread?
//! * TODO: work out the concurrency and parallelism model of the backend, the runtime, and the
//!   compiler and interpreter.
//!
//! ## Data Flow
//!
//! Following the module chain above, here is the data flow between the modules:
//! ```
//!                                                        ,-> Side Effects via Interpretation
//!                                                       /    with the Runtime
//! Source -> Tokens -> AST -> Kernel Sylan -> Sylan IL -<
//!                                                       `-> LLVM IL -> LLVM Target -> Side
//!                                                                                     Effects
//!                                                                                     via Target
//!                                                                                     Executable
//!                                                                                     with the
//!                                                                                     Bundled
//!                                                                                     Runtime
//! ```
//!
//! Source, tokens, and the AST are the usual for programming language implementations. See the
//! `lexing` and `parsing` modules for more information.
//!
//! "Simplification" creates Kernel Sylan from the AST, which is a strict subset of Sylan that
//! strips away conveniences and just exposes the core Sylan semantics. This is the stage from which
//! type checking and Sylan IL creation is performed. This stage still has symbol names and types;
//! it's essentially an AST but stripped down to the language fundamentals.
//!
//! Sylan Intermediate Language has no symbol names except for public, exposed items. It is untyped,
//! therefore the type-checking of Kernel Sylan _must_ have validated before creating Sylan IL from
//! it. It doesn't understand runtime features, expecting IL generation to generate calls out to the
//! runtime functions at the right points. It also is not preemptive, so IL generation must put in
//! yield points correctly to provide ersatz preemptiveness on what is fundamentally a cooperatively
//! scheduled form.
//!
//! TODO: specify precisely how the runtime gets bundled with the compiled artefact. My vague idea
//! currently is to implement it as a Rust module, expose demangled symbols, and then statically
//! link it into the LLVM executable. The interpreter can naturally just invoke it yet another
//! module directly from Rust within the interpreter module.
//!
//! ## Further Details
//!
//! _For more details on each stage, see each modules' documentation._

use std::alloc::System;
use std::env::{args, Args};
use std::fs::File;
use std::io::Read;

use lexing::lexer::Lexer;
use lexing::source::Source;
use lexing::Tokens;
use parsing::Parser;

mod common;
mod lexing;
mod parsing;

#[global_allocator]
static GLOBAL: System = System;

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
