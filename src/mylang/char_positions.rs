use std::{iter::Peekable, str::Chars};

use super::document::Position;

pub struct CharPositions<'a> {
    remaining: Peekable<Chars<'a>>,
    pos: Position,
}

impl CharPositions<'_> {
    pub fn from_str(src: &str) -> CharPositions {
        CharPositions {
            remaining: src.chars().peekable(),
            pos: Position::zero(),
        }
    }
}

impl Iterator for CharPositions<'_> {
    type Item = (Position, char);

    fn next(&mut self) -> Option<Self::Item> {
        self.remaining.next().map(|c| {
            let position = self.pos.clone();

            if c == '\n' {
                self.pos.break_line();
            } else {
                self.pos.advance();
            }

            (position, c)
        })
    }
}
