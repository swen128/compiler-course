use std::{iter::Peekable, vec::IntoIter};

use super::{
    document::Position,
    error::SexpParsingError,
    lexer::{Token, TokenKind},
};

#[derive(Debug)]
pub struct Expr {
    pub kind: ExprKind,
    pub position: Position,
}

#[derive(Debug)]
pub enum ExprKind {
    Atom(Atom),
    List(List),
}

#[derive(Debug)]
pub struct List(pub Vec<Expr>);

#[derive(Debug, PartialEq)]
pub enum Atom {
    Symbol(String),
    Integer(i64),
    Boolean(bool),
    Character(char),
    String(String),
}

impl ExprKind {
    pub fn int(i: i64) -> ExprKind {
        ExprKind::Atom(Atom::Integer(i))
    }

    pub fn bool(b: bool) -> ExprKind {
        ExprKind::Atom(Atom::Boolean(b))
    }

    pub fn symbol(s: &str) -> ExprKind {
        ExprKind::Atom(Atom::Symbol(s.to_string()))
    }

    pub fn char(c: char) -> ExprKind {
        ExprKind::Atom(Atom::Character(c))
    }

    pub fn string(s: &str) -> ExprKind {
        ExprKind::Atom(Atom::String(s.to_string()))
    }

    pub fn list(elems: Vec<Expr>) -> ExprKind {
        ExprKind::List(List(elems))
    }
}

impl Expr {
    pub fn int(i: i64, position: Position) -> Expr {
        Expr {
            kind: ExprKind::int(i),
            position,
        }
    }

    pub fn bool(b: bool, position: Position) -> Expr {
        Expr {
            kind: ExprKind::bool(b),
            position,
        }
    }

    pub fn symbol(s: &str, position: Position) -> Expr {
        Expr {
            kind: ExprKind::symbol(s),
            position,
        }
    }

    pub fn char(c: char, position: Position) -> Expr {
        Expr {
            kind: ExprKind::char(c),
            position,
        }
    }

    pub fn string(s: &str, position: Position) -> Expr {
        Expr {
            kind: ExprKind::string(s),
            position,
        }
    }

    pub fn list(elems: Vec<Expr>, position: Position) -> Expr {
        Expr {
            kind: ExprKind::list(elems),
            position,
        }
    }
}

type Result<T> = std::result::Result<T, SexpParsingError>;

pub fn parse(tokens: Vec<Token>) -> Result<Vec<Expr>> {
    let mut expressions = vec![];
    let mut tokens_iter = tokens.into_iter().peekable();

    while let Some(Token { token: _, position }) = tokens_iter.peek() {
        let position = position.clone();
        let expr = parse_expr(&mut tokens_iter, position)?;
        expressions.push(expr);
    }
    Ok(expressions)
}

fn parse_expr(tokens: &mut Peekable<IntoIter<Token>>, position: Position) -> Result<Expr> {
    match tokens.next() {
        None => Err(err("Unexpected EOF", position)),

        Some(Token { token, position }) => match token {
            TokenKind::ParenOpen => {
                parse_list(tokens, position.clone()).map(|list| Expr::list(list, position))
            }
            TokenKind::ParenClose => Err(err("Unmatched parenthesis ')'", position)),
            TokenKind::Integer(i) => Ok(Expr::int(i, position)),
            TokenKind::Symbol(s) => Ok(Expr::symbol(&s, position)),
            TokenKind::Boolean(b) => Ok(Expr::bool(b, position)),
            TokenKind::Character(c) => Ok(Expr::char(c, position)),
            TokenKind::String(s) => Ok(Expr::string(&s, position)),
        },
    }
}

fn parse_list(tokens: &mut Peekable<IntoIter<Token>>, position: Position) -> Result<Vec<Expr>> {
    let mut list = vec![];
    while let Some(Token { token, position }) = tokens.peek() {
        match token {
            TokenKind::ParenClose => {
                tokens.next();
                return Ok(list);
            }
            _ => {
                let position = position.clone();
                match parse_expr(tokens, position) {
                    Ok(expr) => list.push(expr),
                    Err(e) => return Err(e),
                }
            }
        }
    }
    Err(err("Unmatched parenthesis '('", position))
}

fn err(msg: &str, position: Position) -> SexpParsingError {
    SexpParsingError {
        msg: msg.to_owned(),
        position,
    }
}
