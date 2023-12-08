use super::document::Range;

#[derive(Debug, PartialEq)]
pub enum TokenKind {
    ParenOpen,
    ParenClose,
    Minus,
    Integer(i64),
    Symbol(String),
}

#[derive(Debug)]
pub struct Token {
    pub token: TokenKind,
    pub range: Range,
}
