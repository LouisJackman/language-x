use std::collections::HashMap;

use lexing::tokens::Token;

pub fn new() -> HashMap<&'static str, Token> {
    let mut map = HashMap::new();
    map.extend(vec![
        ("case", Token::Case),
        ("class", Token::Class),
        ("continue", Token::Continue),
        ("default", Token::Default),
        ("do", Token::Do),
        ("else", Token::Else),
        ("extends", Token::Extends),
        ("for", Token::For),
        ("get", Token::Get),
        ("if", Token::If),
        ("implements", Token::Implements),
        ("import", Token::Import),
        ("interface", Token::Interface),
        ("override", Token::Override),
        ("package", Token::Package),
        ("public", Token::Public),
        ("select", Token::Select),
        ("super", Token::Super),
        ("switch", Token::Switch),
        ("throw", Token::Throw),
        ("timeout", Token::Timeout),
        ("_", Token::Ignore),
        ("val", Token::Val),
    ]);
    map
}
