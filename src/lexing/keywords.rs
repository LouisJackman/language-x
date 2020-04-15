//! Keywords are whole words that are lexed as a single unit and reserved
//! by the language.
//!
//! Keywords are reserved to help the parser interpret tokens and resolve
//! ambiguities. Some of the keywords here are reserved but not used. They are
//! reserved to avoid their use as symbols in source files so that they can
//! potentially be used in the future without breaking existing code.

use std::collections::HashMap;

use crate::common::multiphase::{Accessibility, PseudoIdentifier};
use crate::lexing::tokens::{
    Binding, BranchingAndJumping, DeclarationHead, Modifier, ModuleDefinitions, Token,
};

pub fn new() -> HashMap<&'static str, Token> {
    let mut map = HashMap::new();
    map.extend(vec![
        //
        // Pseudoidentifiers
        //
        (
            "_",
            Token::PseudoIdentifier(PseudoIdentifier::PlaceholderIdentifier),
        ),
        (
            "continue",
            Token::PseudoIdentifier(PseudoIdentifier::Continue),
        ),
        ("this", Token::PseudoIdentifier(PseudoIdentifier::This)),
        ("This", Token::PseudoIdentifier(PseudoIdentifier::ThisType)),
        ("it", Token::PseudoIdentifier(PseudoIdentifier::It)),
        ("super", Token::PseudoIdentifier(PseudoIdentifier::Super)),
        //
        // Used
        //
        ("as", Token::Binding(Binding::As)),
        ("class", Token::DeclarationHead(DeclarationHead::Class)),
        (
            "else",
            Token::BranchingAndJumping(BranchingAndJumping::Else),
        ),
        ("embed", Token::Modifier(Modifier::Embed)),
        ("extend", Token::DeclarationHead(DeclarationHead::Extend)),
        ("extends", Token::DeclarationHead(DeclarationHead::Extends)),
        (
            "exports",
            Token::ModuleDefinitions(ModuleDefinitions::Exports),
        ),
        ("extern", Token::Modifier(Modifier::Extern)),
        ("final", Token::Binding(Binding::Final)),
        ("for", Token::BranchingAndJumping(BranchingAndJumping::For)),
        ("fun", Token::DeclarationHead(DeclarationHead::Fun)),
        ("if", Token::BranchingAndJumping(BranchingAndJumping::If)),
        ("ignorable", Token::Modifier(Modifier::Ignorable)),
        ("import", Token::DeclarationHead(DeclarationHead::Import)),
        (
            "implements",
            Token::DeclarationHead(DeclarationHead::Implements),
        ),
        (
            "internal",
            Token::Modifier(Modifier::Accessibility(Accessibility::Internal)),
        ),
        (
            "interface",
            Token::DeclarationHead(DeclarationHead::Interface),
        ),
        ("module", Token::DeclarationHead(DeclarationHead::Module)),
        ("nonvolatile", Token::Modifier(Modifier::NonVolatile)),
        ("operator", Token::Modifier(Modifier::Operator)),
        ("override", Token::Modifier(Modifier::Override)),
        ("package", Token::DeclarationHead(DeclarationHead::Package)),
        (
            "public",
            Token::Modifier(Modifier::Accessibility(Accessibility::Public)),
        ),
        (
            "reject",
            Token::ModuleDefinitions(ModuleDefinitions::Reject),
        ),
        (
            "requires",
            Token::ModuleDefinitions(ModuleDefinitions::Requires),
        ),
        (
            "select",
            Token::BranchingAndJumping(BranchingAndJumping::Select),
        ),
        (
            "switch",
            Token::BranchingAndJumping(BranchingAndJumping::Switch),
        ),
        ("throw", Token::Throw),
        ("timeout", Token::Timeout),
        ("using", Token::Using),
        ("var", Token::Binding(Binding::Var)),
        ("with", Token::With),
        (
            "while",
            Token::BranchingAndJumping(BranchingAndJumping::While),
        ),
        //
        // Reserved, but not yet used
        //
        ("asm", Token::ReservedKeyword),
        ("ast", Token::ReservedKeyword),
        ("alias", Token::ReservedKeyword),
        ("align", Token::ReservedKeyword),
        ("alignto", Token::ReservedKeyword),
        ("arena", Token::ReservedKeyword),
        ("atom", Token::ReservedKeyword),
        ("blittable", Token::ReservedKeyword),
        ("case", Token::ReservedKeyword),
        ("catch", Token::ReservedKeyword),
        ("co", Token::ReservedKeyword),
        ("constexpr", Token::ReservedKeyword),
        ("comptime", Token::ReservedKeyword),
        ("constructor", Token::ReservedKeyword),
        ("checked", Token::ReservedKeyword),
        ("derives", Token::ReservedKeyword),
        ("diverging", Token::ReservedKeyword),
        ("disasm", Token::ReservedKeyword),
        ("do", Token::ReservedKeyword),
        ("dyn", Token::ReservedKeyword),
        ("dynamic", Token::ReservedKeyword),
        ("extern", Token::ReservedKeyword),
        ("fexpr", Token::ReservedKeyword),
        ("fixed", Token::ReservedKeyword),
        ("fun", Token::ReservedKeyword),
        ("forall", Token::ReservedKeyword),
        ("gc", Token::ReservedKeyword),
        ("gen", Token::ReservedKeyword),
        ("get", Token::ReservedKeyword),
        ("infix", Token::ReservedKeyword),
        ("in", Token::ReservedKeyword),
        ("lexemes", Token::ReservedKeyword),
        ("link", Token::ReservedKeyword),
        ("llvm", Token::ReservedKeyword),
        ("macro", Token::ReservedKeyword),
        ("mut", Token::ReservedKeyword),
        ("mutating", Token::ReservedKeyword),
        ("never", Token::ReservedKeyword),
        ("nogc", Token::ReservedKeyword),
        ("noyield", Token::ReservedKeyword),
        ("offset", Token::ReservedKeyword),
        ("offsetof", Token::ReservedKeyword),
        ("pack", Token::ReservedKeyword),
        ("pin", Token::ReservedKeyword),
        ("platform", Token::ReservedKeyword),
        ("prefix", Token::ReservedKeyword),
        ("pragma", Token::ReservedKeyword),
        ("pure", Token::ReservedKeyword),
        ("quasiquote", Token::ReservedKeyword),
        ("quote", Token::ReservedKeyword),
        ("raw", Token::ReservedKeyword),
        ("read", Token::ReservedKeyword),
        ("reader", Token::ReservedKeyword),
        ("ref", Token::ReservedKeyword),
        ("restrict", Token::ReservedKeyword),
        ("stackalloc", Token::ReservedKeyword),
        ("seq", Token::ReservedKeyword),
        ("struct", Token::ReservedKeyword),
        ("source", Token::ReservedKeyword),
        ("sync", Token::ReservedKeyword),
        ("syntax", Token::ReservedKeyword),
        ("throws", Token::ReservedKeyword),
        ("tokens", Token::ReservedKeyword),
        ("total", Token::ReservedKeyword),
        ("transient", Token::ReservedKeyword),
        ("try", Token::ReservedKeyword),
        ("unary", Token::ReservedKeyword),
        ("unchecked", Token::ReservedKeyword),
        ("unsafe", Token::ReservedKeyword),
        ("unquote", Token::ReservedKeyword),
        ("unllvm", Token::ReservedKeyword),
        ("yield", Token::ReservedKeyword),
        ("value", Token::ReservedKeyword),
        ("virtual", Token::ReservedKeyword),
        ("where", Token::ReservedKeyword),
    ]);
    map
}
