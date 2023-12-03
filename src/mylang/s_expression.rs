use std::{iter::Peekable, vec::IntoIter};

use super::tokens::{Token, TokenKind};

pub enum Expr {
    Atom(Atom),
    List(Vec<Expr>),
}

pub enum Atom {
    Symbol(String),
    Integer(i64),
}

pub fn parse(tokens: Vec<Token>) -> Result<Expr, String> {
    // print the tokens for debug
    for token in &tokens {
        println!("{:?}", token);
    }

    let mut tokens_iter = tokens.into_iter().peekable();
    parse_expr(&mut tokens_iter)
}

fn parse_expr(tokens: &mut Peekable<IntoIter<Token>>) -> Result<Expr, String> {
    match tokens.peek() {
        None => Err("Unexpected EOF while parsing exprssion".to_string()),

        Some(Token { token, range }) => match token {
            TokenKind::ParenOpen => parse_list(tokens),
            TokenKind::ParenClose => Err(format!(
                "Unexpected closing parenthesis at: {:?}",
                range.start
            )),
            TokenKind::Integer(_) => parse_atom(tokens),
            TokenKind::Symbol(_) => parse_atom(tokens),
        },
    }
}

fn parse_list(tokens: &mut Peekable<IntoIter<Token>>) -> Result<Expr, String> {
    let mut list = vec![];
    tokens.next();
    while let Some(Token { token, range: _ }) = tokens.peek() {
        match token {
            TokenKind::ParenClose => {
                tokens.next();
                return Ok(Expr::List(list));
            }
            _ => match parse_expr(tokens) {
                Ok(expr) => list.push(expr),
                Err(e) => return Err(e),
            },
        }
    }
    Err("Unexpected EOF while parsing list".to_string())
}

fn parse_atom(tokens: &mut Peekable<IntoIter<Token>>) -> Result<Expr, String> {
    match tokens.next() {
        None => Err("Unexpected EOF while parsing atom".to_string()),

        Some(Token { token, range }) => match token {
            TokenKind::Integer(i) => Ok(Expr::Atom(Atom::Integer(i))),
            TokenKind::Symbol(s) => Ok(Expr::Atom(Atom::Symbol(s))),
            _ => Err(format!("Unexpected token at: {:?}", range.start)),
        },
    }
}
