use std::collections::HashMap;

pub fn new() -> HashMap<char, char> {
    let mut map = HashMap::new();
    map.extend(vec![
        ('n', '\n'),
        ('r', '\r'),
        ('t', '\t'),
        ('\\', '\\'),
        ('\'', '\''),
    ]);
    map
}
