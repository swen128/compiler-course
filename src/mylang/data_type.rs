use super::ast::Lit;

#[allow(dead_code)]
pub enum Value {
    Int(i64),
    Byte(u8),
    Boolean(bool),
    Char(char),
    Eof,
    Void,
}

impl Value {
    pub fn encode(self) -> i64 {
        match self {
            Value::Int(i) => i << INT_SHIFT,
            Value::Byte(b) => (b as i64) << INT_SHIFT,
            Value::Char(c) => ((c as i64) << CHAR_SHIFT) + TYPE_CHAR,
            Value::Boolean(true) => TYPE_TRUE,
            Value::Boolean(false) => TYPE_FALSE,
            Value::Eof => TYPE_EOF,
            Value::Void => TYPE_VOID,
        }
    }
}

impl From<Lit> for Value {
    fn from(value: Lit) -> Self {
        match value {
            Lit::Int(i) => Value::Int(i),
            Lit::Char(c) => Value::Char(c),
            Lit::Bool(b) => Value::Boolean(b),
        }
    }
}

pub const INT_SHIFT: i64 = 1;
pub const MASK_INT: i64 = 0b1;
pub const TYPE_INT: i64 = 0b0;

pub const CHAR_SHIFT: i64 = 2;
pub const TYPE_CHAR: i64 = 0b01;
pub const MASK_CHAR: i64 = 0b11;

const TYPE_TRUE: i64 = 0b11;
const TYPE_FALSE: i64 = 0b111;

const TYPE_EOF: i64 = 0b1011;

const TYPE_VOID: i64 = 0b1111;
