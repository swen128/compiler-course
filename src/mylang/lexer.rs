use std::{
    char,
    iter::{Enumerate, Peekable},
    str::Chars,
};

use super::{
    document::Range,
    tokens::{Token, TokenKind},
};

struct Tokenizer<'a> {
    remaining_chars: Peekable<Enumerate<Chars<'a>>>,
}

impl<'a> Tokenizer<'a> {
    fn new(src: &str) -> Tokenizer {
        Tokenizer {
            remaining_chars: src.chars().enumerate().peekable(),
        }
    }

    fn next_token(&mut self) -> Result<Option<Token>, String> {
        self.skip_whitespaces();
        
        match self.remaining_chars.peek() {
            None => Ok(None),

            Some((i, _)) => {
                let start = *i;

                let result = take_first_token(&mut self.remaining_chars);

                result.map(|(kind, len)| {
                    let end = start + len;
                    let range = Range { start, end };
                    Some(Token { token: kind, range })
                })
            }
        }
    }

    fn skip_whitespaces(&mut self) {
        while let Some(_) = self.remaining_chars.next_if(|(_, c)| c.is_whitespace()) {}
    }
}

pub fn tokenize(src: &str) -> Result<Vec<Token>, String> {
    let mut tokenizer = Tokenizer::new(src);
    let mut tokens = Vec::new();

    while let Some(tok) = tokenizer.next_token()? {
        tokens.push(tok);
    }

    Ok(tokens)
}

fn take_first_token(chars: &mut Peekable<Enumerate<Chars>>) -> Result<(TokenKind, usize), String> {
    match chars.peek() {
        None => return Err("Unexpected end of input".to_string()),
        Some((_, char)) => match char {
            '(' => {
                chars.next();
                Ok((TokenKind::ParenOpen, 1))
            }
            ')' => {
                chars.next();
                Ok((TokenKind::ParenClose, 1))
            }
            '-' => {
                chars.next();
                Ok((TokenKind::Minus, 1))
            }
            '1'..='9' => Ok(take_int_token(chars)),
            _ if is_symbol_char(char) => Ok(take_symbol_token(chars)),
            _ => Err(format!("Unexpected character: {}", char)),
        },
    }
}

fn take_int_token(chars: &mut Peekable<Enumerate<Chars>>) -> (TokenKind, usize) {
    let mut char_count = 0;
    
    let mut digits = "".to_string();
    while let Some((_, c)) = chars.peek() {
        if !c.is_digit(10) {
            break;
        }
        digits.push(*c);
        chars.next();
        char_count += 1;
    }
    
    let value = digits.parse::<i64>().unwrap();

    (TokenKind::Integer(value), char_count)
}

fn take_symbol_token(chars: &mut Peekable<Enumerate<Chars>>) -> (TokenKind, usize) {
    let mut char_count = 0;
    
    let mut symbol = "".to_string();
    while let Some((_, c)) = chars.peek() {
        if !is_symbol_char(c) {
            break;
        }
        symbol.push(*c);
        chars.next();
        char_count += 1;
    }

    (TokenKind::Symbol(symbol), char_count)
}

fn is_symbol_char(char: &char) -> bool {
    char.is_alphanumeric() || *char == '_'
}
