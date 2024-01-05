pub enum Value {
    Int(i64),
    Byte(u8),
    Boolean(bool),
    Char(char),
    Eof,
    Void,
    EmptyList,
    EmptyVector,
    EmptyString,
    Cons(Address),
    Box(Address),
    Vector(Address),
    String(Address),
}

pub struct Address(pub i64);

impl Value {
    pub fn encode(self) -> i64 {
        match self {
            Value::Int(i) => INT_TYPE.encode(i),
            Value::Byte(b) => INT_TYPE.encode(b as i64),
            Value::Char(c) => CHAR_TYPE.encode(c as i64),

            // TODO: Memory address is only availabe at runtime, so we never use these.
            Value::Box(Address(a)) => BOX_TYPE.encode(a),
            Value::Cons(Address(a)) => CONS_TYPE.encode(a),
            Value::Vector(Address(a)) => VECTOR_TYPE.encode(a),
            Value::String(Address(a)) => STRING_TYPE.encode(a),

            Value::Boolean(true) => TRUE_TYPE.0 as i64,
            Value::Boolean(false) => FALSE_TYPE.0 as i64,
            Value::Eof => EOF_TYPE.0 as i64,
            Value::Void => VOID_TYPE.0 as i64,
            Value::EmptyList => EMPTY_TYPE.0 as i64,
            Value::EmptyVector => VECTOR_TYPE.tag.0 as i64,
            Value::EmptyString => STRING_TYPE.tag.0 as i64,
        }
    }
}

pub struct TypeTag(pub u64);

pub struct UnaryType {
    pub shift: u64,
    pub tag: TypeTag,
}

impl UnaryType {
    pub fn encode(&self, value: i64) -> i64 {
        (value << self.shift) + (self.tag.0 as i64)
    }

    pub fn mask(&self) -> u64 {
        (1 << self.shift) - 1
    }
}

// Bit layout of values
//
// Values are either:
// - Immediates: end in #b000
// - Pointers
//
// Pointers are either:
// - Boxes:   end in #b001
// - Cons:    end in #b010
// - Vector:  end in #b011
// - String:  end in #b100
// - Closure: end in #b101
//
// Immediates are either
// - Integers:   end in  #b0 000
// - Characters: end in #b01 000
// - True:              #b11 000
// - False:           #b1 11 000
// - Eof:            #b10 11 000
// - Void:           #b11 11 000
// - Empty:         #b100 11 000
const IMMEDIATE_SHIFT: u64 = 3;

pub const BOX_TYPE: UnaryType = UnaryType {
    shift: IMMEDIATE_SHIFT,
    tag: TypeTag(0b001),
};

pub const CONS_TYPE: UnaryType = UnaryType {
    shift: IMMEDIATE_SHIFT,
    tag: TypeTag(0b010),
};

pub const VECTOR_TYPE: UnaryType = UnaryType {
    shift: IMMEDIATE_SHIFT,
    tag: TypeTag(0b011),
};

pub const STRING_TYPE: UnaryType = UnaryType {
    shift: IMMEDIATE_SHIFT,
    tag: TypeTag(0b100),
};

pub const CLOSURE_TYPE: UnaryType = UnaryType {
    shift: IMMEDIATE_SHIFT,
    tag: TypeTag(0b101),
};

pub const INT_TYPE: UnaryType = UnaryType {
    shift: 1 + IMMEDIATE_SHIFT,
    tag: TypeTag(0b0 << IMMEDIATE_SHIFT),
};

pub const CHAR_TYPE: UnaryType = UnaryType {
    shift: 2 + IMMEDIATE_SHIFT,
    tag: TypeTag(0b01 << IMMEDIATE_SHIFT),
};

pub const TRUE_TYPE: TypeTag = TypeTag(0b11 << IMMEDIATE_SHIFT);
pub const FALSE_TYPE: TypeTag = TypeTag(0b111 << IMMEDIATE_SHIFT);
pub const EOF_TYPE: TypeTag = TypeTag(0b1011 << IMMEDIATE_SHIFT);
pub const VOID_TYPE: TypeTag = TypeTag(0b1111 << IMMEDIATE_SHIFT);
pub const EMPTY_TYPE: TypeTag = TypeTag(0b10011 << IMMEDIATE_SHIFT);
