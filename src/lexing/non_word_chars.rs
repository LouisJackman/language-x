use std::collections::HashSet;

pub fn new() -> HashSet<char> {
    let mut non_word_chars = HashSet::new();
    non_word_chars.extend(vec![';', '.', ',', '{', '}', '(', ')', '[', ']']);
    non_word_chars
}
