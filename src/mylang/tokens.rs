use super::document::Position;
use logos::Logos;

#[derive(Logos, Debug, PartialEq)]
#[logos(skip r"[ \t\n\f]+")]
pub enum TokenKind {
    #[token("(")]
    ParenOpen,

    #[token(")")]
    ParenClose,

    #[token("-")]
    Minus,

    #[regex(r"[0-9]+", |lex| lex.slice().parse().ok())]
    Integer(i64),

    #[regex(r"[^ \t\n\f()0-9][^ \t\n\f()]*", |lex| lex.slice().to_string())]
    Symbol(String),

    #[regex(r"#[tf]", |lex| parse_bool(lex.slice()))]
    Boolean(bool),
    
    #[regex(r"#\\.", |lex| lex.slice().chars().nth(2).unwrap())]
    Character(char),
}

fn parse_bool(str: &str) -> bool {
    match str {
        "#t" => true,
        "#f" => false,
        _ => panic!("Invalid boolean literal"),
    }
}


#[derive(Debug)]
pub struct Token {
    pub token: TokenKind,
    pub position: Position,
}
