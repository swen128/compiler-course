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
}

impl Expr {
    pub fn int(i: i64, position: Position) -> Expr {
        Expr {
            kind: ExprKind::Atom(Atom::Integer(i)),
            position,
        }
    }

    pub fn bool(b: bool, position: Position) -> Expr {
        Expr {
            kind: ExprKind::Atom(Atom::Boolean(b)),
            position,
        }
    }

    pub fn symbol(s: &str, position: Position) -> Expr {
        Expr {
            kind: ExprKind::Atom(Atom::Symbol(s.to_string())),
            position,
        }
    }

    pub fn char(c: char, position: Position) -> Expr {
        Expr {
            kind: ExprKind::Atom(Atom::Character(c)),
            position,
        }
    }

    pub fn list(elems: Vec<Expr>, position: Position) -> Expr {
        Expr {
            kind: ExprKind::List(List(elems)),
            position,
        }
    }
}

type Result<T> = std::result::Result<T, SexpParsingError>;

pub fn parse(tokens: Vec<Token>) -> Result<Expr> {
    // print the tokens for debug
    for token in &tokens {
        println!("{:?}", token);
    }

    let mut tokens_iter = tokens.into_iter().peekable();
    parse_expr(&mut tokens_iter, Position::zero())
}

fn parse_expr(tokens: &mut Peekable<IntoIter<Token>>, position: Position) -> Result<Expr> {
    match tokens.next() {
        None => Err(err("Unexpected EOF", position)),

        Some(Token { token, position }) => match token {
            TokenKind::ParenOpen => {
                parse_list(tokens, position.clone()).map(|list| Expr::list(list, position))
            }
            TokenKind::ParenClose => Err(err("Unmatched parenthesis ')'", position)),
            TokenKind::Minus => parse_negative_int(tokens, position),
            TokenKind::Integer(i) => Ok(Expr::int(i, position)),
            TokenKind::Symbol(s) => Ok(Expr::symbol(&s, position)),
            TokenKind::Boolean(b) => Ok(Expr::bool(b, position)),
            TokenKind::Character(c) => Ok(Expr::char(c, position)),
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

fn parse_negative_int(tokens: &mut Peekable<IntoIter<Token>>, position: Position) -> Result<Expr> {
    match tokens.next() {
        Some(Token {
            token: TokenKind::Integer(i),
            position,
        }) => Ok(Expr::int(-i, position)),

        Some(Token { token, position }) => Err(err(
            &format!("Expected an integer. Got {:?}", token),
            position,
        )),

        None => Err(err(
            "Expected an integer after minus sign. Got EOF instead.",
            position,
        )),
    }
}

fn err(msg: &str, position: Position) -> SexpParsingError {
    SexpParsingError {
        msg: msg.to_owned(),
        position,
    }
}
