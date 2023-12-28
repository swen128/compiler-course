pub enum Value {
    Int(i64),
    Boolean(bool),
    Char(char),
    Eof,
    Void,
}

impl Value {
    pub fn encode(self) -> i64 {
        match self {
            Value::Int(i) => i << INT_SHIFT,
            Value::Char(c) => ((c as i64) << CHAR_SHIFT) + TYPE_CHAR,
            Value::Boolean(true) => 0b11,
            Value::Boolean(false) => 0b111,
            Value::Eof => 0b1011,
            Value::Void => 0b1111,
        }
    }
}

pub const INT_SHIFT: i64 = 1;
pub const CHAR_SHIFT: i64 = 2;
pub const TYPE_CHAR: i64 = 0b01;
pub const TYPE_INT: i64 = 0b0;
pub const MASK_CHAR: i64 = 0b11;
pub const MASK_INT: i64 = 0b1;
