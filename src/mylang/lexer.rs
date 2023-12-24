use std::{char, iter::Peekable};

use super::{
    char_positions::CharPositions,
    error::InvalidTokenError,
    tokens::{Token, TokenKind},
};

struct Tokenizer<'a> {
    remaining_chars: Peekable<CharPositions<'a>>,
}

impl<'a> Tokenizer<'a> {
    fn new(src: &str) -> Tokenizer {
        Tokenizer {
            remaining_chars: CharPositions::from_str(src).peekable(),
        }
    }

    fn next_token(&mut self) -> Result<Option<Token>, InvalidTokenError> {
        self.skip_whitespaces();

        match self.remaining_chars.next() {
            None => Ok(None),

            Some((position, char)) => {
                let position = position.clone();

                match take_first_token(&char, &mut self.remaining_chars) {
                    Ok(token) => Ok(Some(Token { token, position })),
                    Err(_) => Err(InvalidTokenError { position }),
                }
            }
        }
    }

    fn skip_whitespaces(&mut self) {
        while let Some(_) = self.remaining_chars.next_if(|(_, c)| c.is_whitespace()) {}
    }
}

pub fn tokenize(src: &str) -> Result<Vec<Token>, InvalidTokenError> {
    let mut tokenizer = Tokenizer::new(src);
    let mut tokens = Vec::new();

    while let Some(tok) = tokenizer.next_token()? {
        tokens.push(tok);
    }

    Ok(tokens)
}

fn take_first_token(
    head: &char,
    tail: &mut Peekable<CharPositions>,
) -> Result<TokenKind, InvalidCharacter> {
    match head {
        '(' => Ok(TokenKind::ParenOpen),
        ')' => Ok(TokenKind::ParenClose),
        '-' => Ok(TokenKind::Minus),
        '#' => take_hash_token(tail),
        '0'..='9' => Ok(take_int_token(head, tail)),
        _ if is_symbol_char(head) => Ok(take_symbol_token(head, tail)),
        _ => Err(InvalidCharacter {}),
    }
}

fn take_int_token(head: &char, tail: &mut Peekable<CharPositions>) -> TokenKind {
    let mut digits = head.to_string();
    while let Some((_, c)) = tail.peek() {
        if !c.is_digit(10) {
            break;
        }
        digits.push(*c);
        tail.next();
    }

    let value = digits.parse::<i64>().unwrap();

    TokenKind::Integer(value)
}

fn take_symbol_token(head: &char, chars: &mut Peekable<CharPositions>) -> TokenKind {
    let mut symbol = head.to_string();
    while let Some((_, c)) = chars.peek() {
        if !is_symbol_char(c) {
            break;
        }
        symbol.push(*c);
        chars.next();
    }

    TokenKind::Symbol(symbol)
}

fn take_hash_token(chars: &mut Peekable<CharPositions>) -> Result<TokenKind, InvalidCharacter> {
    match chars.next() {
        Some((_, 't')) => Ok(TokenKind::Boolean(true)),
        Some((_, 'f')) => Ok(TokenKind::Boolean(false)),
        Some((_, '\\')) => match chars.next() {
            Some((_, c)) => Ok(TokenKind::Character(c)),
            None => Err(InvalidCharacter {}),
        },
        _ => Err(InvalidCharacter {}),
    }
}

fn is_symbol_char(char: &char) -> bool {
    !char.is_control() && !char.is_whitespace()
}

struct InvalidCharacter {}
