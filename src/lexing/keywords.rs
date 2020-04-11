//! Keywords are reserved keywords that help the parser interpret tokens and resolve ambiguities.
//! Some of the "keywords" here are actually just reserved words. They are reserved to avoid their
//! use as symbols in source files so that they can potentially be used in the future without
//! breaking existing code.

use std::collections::HashMap;

use crate::common::multiphase::{Accessibility, PseudoIdentifier};
use crate::lexing::tokens::{
    Binding, BranchingAndJumping, DeclarationHead, Modifier, ModuleDefinitions, Token,
};

pub fn new() -> HashMap<&'static str, Token> {
    let mut map = HashMap::new();
    map.extend(vec![
        (
            "_",
            Token::PseudoIdentifier(PseudoIdentifier::PlaceholderIdentifier),
        ),
        ("as", Token::Binding(Binding::As)),
        ("class", Token::DeclarationHead(DeclarationHead::Class)),
        (
            "continue",
            Token::PseudoIdentifier(PseudoIdentifier::Continue),
        ),
        (
            "else",
            Token::BranchingAndJumping(BranchingAndJumping::Else),
        ),
        ("embed", Token::Modifier(Modifier::Embed)),
        ("extend", Token::DeclarationHead(DeclarationHead::Extend)),
        (
            "exports",
            Token::ModuleDefinitions(ModuleDefinitions::Exports),
        ),
        ("extern", Token::Modifier(Modifier::Extern)),
        ("final", Token::Binding(Binding::Final)),
        ("for", Token::BranchingAndJumping(BranchingAndJumping::For)),
        ("fun", Token::DeclarationHead(DeclarationHead::Fun)),
        ("if", Token::BranchingAndJumping(BranchingAndJumping::If)),
        ("it", Token::PseudoIdentifier(PseudoIdentifier::It)),
        ("ignorable", Token::Modifier(Modifier::Ignorable)),
        ("import", Token::DeclarationHead(DeclarationHead::Import)),
        (
            "internal",
            Token::Modifier(Modifier::Accessibility(Accessibility::Internal)),
        ),
        (
            "interface",
            Token::DeclarationHead(DeclarationHead::Interface),
        ),
        ("module", Token::DeclarationHead(DeclarationHead::Module)),
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
        ("super", Token::PseudoIdentifier(PseudoIdentifier::Super)),
        (
            "switch",
            Token::BranchingAndJumping(BranchingAndJumping::Switch),
        ),
        ("this", Token::PseudoIdentifier(PseudoIdentifier::This)),
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
        // Reserved but not used.
        //
        ("asm", Token::ReservedKeyword),
        ("alias", Token::ReservedKeyword),
        ("align", Token::ReservedKeyword),
        ("arena", Token::ReservedKeyword),
        ("atom", Token::ReservedKeyword),
        ("blittable", Token::ReservedKeyword),
        ("case", Token::ReservedKeyword),
        ("catch", Token::ReservedKeyword),
        ("co", Token::ReservedKeyword),
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
        ("implements", Token::ReservedKeyword),
        ("in", Token::ReservedKeyword),
        ("link", Token::ReservedKeyword),
        ("llvm", Token::ReservedKeyword),
        ("macro", Token::ReservedKeyword),
        ("mut", Token::ReservedKeyword),
        ("mutating", Token::ReservedKeyword),
        ("never", Token::ReservedKeyword),
        ("nogc", Token::ReservedKeyword),
        ("noyield", Token::ReservedKeyword),
        ("pack", Token::ReservedKeyword),
        ("pin", Token::ReservedKeyword),
        ("platform", Token::ReservedKeyword),
        ("prefix", Token::ReservedKeyword),
        ("pragma", Token::ReservedKeyword),
        ("pure", Token::ReservedKeyword),
        ("quasiquote", Token::ReservedKeyword),
        ("quote", Token::ReservedKeyword),
        ("reader", Token::ReservedKeyword),
        ("ref", Token::ReservedKeyword),
        ("stackalloc", Token::ReservedKeyword),
        ("seq", Token::ReservedKeyword),
        ("struct", Token::ReservedKeyword),
        ("sync", Token::ReservedKeyword),
        ("throws", Token::ReservedKeyword),
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
