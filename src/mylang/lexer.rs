use std::{char, iter::Peekable};

use super::{
    char_positions::CharPositions,
    document::Position,
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

    fn next_char(&mut self) -> Option<(Position, char)> {
        self.remaining_chars.next()
    }

    fn peek(&mut self) -> Option<&(Position, char)> {
        self.remaining_chars.peek()
    }

    fn next_token(&mut self) -> Result<Option<Token>, String> {
        self.skip_whitespaces();

        match self.peek() {
            None => Ok(None),

            Some((position, _)) => {
                let start = position.clone();
                let result = take_first_token(&mut self.remaining_chars);
                result.map(move |token| {
                    Some(Token {
                        token,
                        position: start,
                    })
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

fn take_first_token(chars: &mut Peekable<CharPositions>) -> Result<TokenKind, String> {
    match chars.peek() {
        None => return Err("Unexpected end of input".to_string()),
        Some((_, char)) => match char {
            '(' => {
                chars.next();
                Ok(TokenKind::ParenOpen)
            }
            ')' => {
                chars.next();
                Ok(TokenKind::ParenClose)
            }
            '-' => {
                chars.next();
                Ok(TokenKind::Minus)
            }
            '#' => take_boolean_token(chars),
            '0'..='9' => Ok(take_int_token(chars)),
            _ if char.is_alphanumeric() => Ok(take_symbol_token(chars)),
            _ => Err(format!("Unexpected character: {}", char)),
        },
    }
}

fn take_int_token(chars: &mut Peekable<CharPositions>) -> TokenKind {
    let mut digits = "".to_string();
    while let Some((_, c)) = chars.peek() {
        if !c.is_digit(10) {
            break;
        }
        digits.push(*c);
        chars.next();
    }

    let value = digits.parse::<i64>().unwrap();

    TokenKind::Integer(value)
}

fn take_symbol_token(chars: &mut Peekable<CharPositions>) -> TokenKind {
    let mut symbol = "".to_string();
    while let Some((_, c)) = chars.peek() {
        if !is_symbol_char(c) {
            break;
        }
        symbol.push(*c);
        chars.next();
    }

    TokenKind::Symbol(symbol)
}

fn take_boolean_token(chars: &mut Peekable<CharPositions>) -> Result<TokenKind, String> {
    // TODO: Check that the first character is '#'.
    chars.next();

    match chars.next() {
        Some((_, 't')) => Ok(TokenKind::Boolean(true)),
        Some((_, 'f')) => Ok(TokenKind::Boolean(false)),
        Some((pos, c)) => Err(format!(
            "Expected a boolean token. Got '#{}' instead at position {:?}.",
            c, pos
        )),
        None => Err("Expectead a boolean token. Got EOF instead.".to_string()),
    }
}

fn is_symbol_char(char: &char) -> bool {
    char.is_alphanumeric() || *char == '?' || *char == '!' || *char == '_'
}
