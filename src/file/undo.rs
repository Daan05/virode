use std::time::Instant;

#[derive(Debug, Clone, Copy)]
pub struct CursorPos {
    pub row: u16,
    pub col: u16,
}

#[derive(Debug, Clone, Copy)]
pub struct TermSize {
    pub width: u16,
    pub height: u16,
}

#[derive(Debug)]
pub struct Snapshot {
    pub content: Vec<String>,
    pub cursor: CursorPos,
    pub timestamp: Instant,
}
