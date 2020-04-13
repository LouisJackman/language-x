#[derive(Eq, PartialEq)]
pub enum NewLine {
    // Unix
    LineFeed,

    // Windows
    CarridgeReturnLineFeed,

    // Classic MacOS
    CarridgeReturn,
}

pub fn check_newline(current: char, next: Option<char>) -> Option<NewLine> {
    if current == '\n' {
        Some(NewLine::LineFeed)
    } else if current == '\r' {
        if next.filter(|&c| c == '\n').is_some() {
            Some(NewLine::CarridgeReturnLineFeed)
        } else {
            Some(NewLine::CarridgeReturn)
        }
    } else {
        None
    }
}
