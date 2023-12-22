#[derive(Debug)]
pub struct Range {
    pub start: Position,
    pub end: Position,
}

#[derive(Debug, Clone)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}

impl Position {
    pub fn zero() -> Position {
        Position { line: 0, column: 0 }
    }

    pub fn new(line: usize, column: usize) -> Position {
        Position { line, column }
    }

    pub fn break_line(&mut self) {
        self.line += 1;
        self.column = 0;
    }

    pub fn advance(&mut self) {
        self.column += 1;
    }
}
