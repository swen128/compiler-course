#[derive(Debug)]
pub struct Range {
    pub start: Position,
    pub end: Position,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Position {
    pub offset: usize,
}

impl Position {
    pub fn zero() -> Position {
        Position { offset: 0 }
    }

    pub fn new(offset: usize) -> Position {
        Position { offset }
    }
}

impl std::fmt::Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.offset)
    }
}
