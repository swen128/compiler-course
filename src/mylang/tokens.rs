use super::document::Position;

#[derive(Debug, PartialEq)]
pub enum TokenKind {
    ParenOpen,
    ParenClose,
    Minus,
    Integer(i64),
    Symbol(String),
    Boolean(bool),
    Character(char),
}

#[derive(Debug)]
pub struct Token {
    pub token: TokenKind,
    pub position: Position,
}
