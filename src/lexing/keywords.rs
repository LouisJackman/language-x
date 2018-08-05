use std::collections::HashMap;

use lexing::tokens::Token;

pub fn new() -> HashMap<&'static str, Token> {
    let mut map = HashMap::new();
    map.extend(vec![
        ("_", Token::Ignore),
        ("class", Token::Class),
        ("continue", Token::Continue),
        ("default", Token::Default),
        ("do", Token::Do),
        ("else", Token::Else),
        ("embeds", Token::Embeds),
        ("extend", Token::Extend),
        ("extends", Token::Extends),
        ("for", Token::For),
        ("get", Token::Get),
        ("if", Token::If),
        ("implements", Token::Implements),
        ("import", Token::Import),
        ("interface", Token::Interface),
        ("override", Token::Override),
        ("package", Token::Package),
        ("pure", Token::Pure),
        ("select", Token::Select),
        ("super", Token::Super),
        ("switch", Token::Switch),
        ("throw", Token::Throw),
        ("total", Token::Total),
        ("timeout", Token::Timeout),
        ("var", Token::Var),
        ("where", Token::Where),
    ]);
    map
}
