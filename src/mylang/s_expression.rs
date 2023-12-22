use std::{iter::Peekable, vec::IntoIter};

use super::tokens::{Token, TokenKind};

#[derive(Debug)]
pub enum Expr {
    Atom(Atom),
    List(List),
}

#[derive(Debug)]
pub struct List {
    pub head: Atom,
    pub tail: Vec<Expr>,
}

#[derive(Debug, PartialEq)]
pub enum Atom {
    Symbol(String),
    Integer(i64),
    Boolean(bool),
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

        Some(Token { token, position }) => match token {
            TokenKind::ParenOpen => parse_list(tokens),
            TokenKind::ParenClose => {
                Err(format!("Unexpected closing parenthesis at: {:?}", position,))
            }
            TokenKind::Minus => parse_negative_int(tokens),
            TokenKind::Integer(_) => parse_positive_int(tokens),
            TokenKind::Symbol(_) => parse_symbol(tokens),
            TokenKind::Boolean(_) => parse_boolean(tokens),
        },
    }
}

fn parse_list(tokens: &mut Peekable<IntoIter<Token>>) -> Result<Expr, String> {
    tokens
        .next()
        .ok_or("Expected opening parenthesis. Got EOF instead.")?;

    let head = match tokens.next() {
        Some(Token { token, position: _ }) => match token {
            TokenKind::Symbol(s) => Atom::Symbol(s),
            _ => return Err("Expected operator".to_string()),
        },
        None => return Err("Unexpected EOF while parsing list".to_string()),
    };
    let tail = parse_list_rest(tokens)?;

    Ok(Expr::List(List { head, tail }))
}

fn parse_list_rest(tokens: &mut Peekable<IntoIter<Token>>) -> Result<Vec<Expr>, String> {
    let mut list = vec![];
    while let Some(Token { token, position: _ }) = tokens.peek() {
        match token {
            TokenKind::ParenClose => {
                tokens.next();
                return Ok(list);
            }
            _ => match parse_expr(tokens) {
                Ok(expr) => list.push(expr),
                Err(e) => return Err(e),
            },
        }
    }
    Err("Unexpected EOF while parsing list".to_string())
}

fn parse_symbol(tokens: &mut Peekable<IntoIter<Token>>) -> Result<Expr, String> {
    match tokens.next() {
        None => Err("Unexpected EOF while parsing symbol".to_string()),

        Some(Token { token, position }) => match token {
            TokenKind::Symbol(s) => Ok(Expr::Atom(Atom::Symbol(s))),
            _ => Err(format!("Unexpected token at: {:?}", position)),
        },
    }
}

fn parse_positive_int(tokens: &mut Peekable<IntoIter<Token>>) -> Result<Expr, String> {
    match tokens.next() {
        None => Err("Unexpected EOF while parsing positive int".to_string()),

        Some(Token { token, position }) => match token {
            TokenKind::Integer(i) => Ok(Expr::Atom(Atom::Integer(i))),
            _ => Err(format!("Unexpected token at: {:?}", position)),
        },
    }
}

fn parse_negative_int(tokens: &mut Peekable<IntoIter<Token>>) -> Result<Expr, String> {
    let Token { token, position } = tokens
        .next()
        .ok_or("Unexpected EOF while parsing negative int")?;

    if token != TokenKind::Minus {
        return Err(format!("Unexpected token '{:?}' at: {:?}", token, position));
    }

    let Token { token, position } = tokens
        .next()
        .ok_or("Unexpected EOF while parsing negative int")?;

    match token {
        TokenKind::Integer(i) => Ok(Expr::Atom(Atom::Integer(-i))),
        _ => Err(format!("Unexpected token at: {:?}", position)),
    }
}

fn parse_boolean(tokens: &mut Peekable<IntoIter<Token>>) -> Result<Expr, String> {
    match tokens.next() {
        None => Err("Unexpected EOF while parsing boolean".to_string()),

        Some(Token { token, position }) => match token {
            TokenKind::Boolean(b) => Ok(Expr::Atom(Atom::Boolean(b))),
            _ => Err(format!("Unexpected token at: {:?}", position)),
        },
    }
}
