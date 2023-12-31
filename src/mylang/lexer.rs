use super::error::InvalidTokenError;
use crate::Position;
use logos::{Lexer, Logos};

#[derive(Logos, Debug, PartialEq)]
#[logos(skip r"\s+")]
pub enum TokenKind {
    #[token("(")]
    ParenOpen,

    #[token(")")]
    ParenClose,

    #[token("-")]
    Minus,

    #[regex(r"[0-9]+", |lex| lex.slice().parse().ok())]
    Integer(i64),

    #[regex(r"[^\s\-()0-9][^\s()]*", |lex| lex.slice().to_string())]
    Symbol(String),

    #[regex(r"#[tf]", parse_bool)]
    Boolean(bool),

    #[regex(r"#\\.", |lex| lex.slice().chars().nth(2).unwrap())]
    Character(char),

    #[regex(r#""(?:[^"]|\\")*""#, strip_first_and_last_char)]
    String(String),
}

#[derive(Debug)]
pub struct Token {
    pub token: TokenKind,
    pub position: Position,
}

pub fn tokenize(src: &str) -> Result<Vec<Token>, InvalidTokenError> {
    let results = TokenKind::lexer(src).spanned();

    results
        .map(|(result, span)| match result {
            Ok(token) => Ok(Token {
                token,
                position: Position::new(span.start),
            }),

            Err(_) => Err(InvalidTokenError {
                position: Position { offset: span.start },
            }),
        })
        .collect()
}

fn parse_bool(lex: &mut Lexer<TokenKind>) -> bool {
    match lex.slice() {
        "#t" => true,
        "#f" => false,
        _ => panic!("Invalid boolean literal"),
    }
}

fn strip_first_and_last_char(lex: &mut Lexer<TokenKind>) -> String {
    let mut chars = lex.slice().chars();
    chars.next();
    chars.next_back();
    chars.collect()
}
