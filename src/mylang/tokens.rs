use super::document::Range;

#[derive(Debug)]
pub enum TokenKind {
    ParenOpen,
    ParenClose,
    Integer(i64),
    Symbol(String),
}

#[derive(Debug)]
pub struct Token {
    pub token: TokenKind,
    pub range: Range,
}
