use crate::{
    a86::ast::{Operand, Register, Statement},
    mylang::{compiler::error::ERR_LABEL, data_type::*},
};

use super::helper::if_equal;

const RAX: Operand = Operand::Register(Register::RAX);
const R9: Operand = Operand::Register(Register::R9);

pub fn cast_type(register: Register, from: &UnaryType, to: &UnaryType) -> Vec<Statement> {
    vec![
        Statement::Sar {
            dest: Operand::Register(register.clone()),
            src: Operand::Immediate(from.shift as i64),
        },
        Statement::Sal {
            dest: Operand::Register(register.clone()),
            src: Operand::Immediate(to.shift as i64),
        },
        Statement::Xor {
            dest: Operand::Register(register),
            src: Operand::Immediate(to.tag.0 as i64),
        },
    ]
}

/// Returns instructions which sets rax to true iff the value in rax is of the given type.
pub fn is_type(type_: &UnaryType) -> Vec<Statement> {
    let mut statements = vec![
        Statement::And {
            dest: RAX,
            src: Operand::Immediate(type_.mask() as i64),
        },
        Statement::Cmp {
            dest: RAX,
            src: Operand::Immediate(type_.tag.0 as i64),
        },
    ];
    statements.extend(if_equal());
    statements
}

pub fn is_eof() -> Vec<Statement> {
    let mut statements = vec![Statement::Cmp {
        dest: RAX,
        src: Operand::Immediate(Value::Eof.encode()),
    }];
    statements.extend(if_equal());
    statements
}

pub fn char_to_int() -> Vec<Statement> {
    let mut statements = assert_char(Register::RAX);
    statements.extend(cast_type(Register::RAX, &CHAR_TYPE, &INT_TYPE));
    statements
}

pub fn int_to_char() -> Vec<Statement> {
    let mut statements = assert_int(Register::RAX);
    statements.extend(assert_codepoint());
    statements.extend(cast_type(Register::RAX, &INT_TYPE, &CHAR_TYPE));
    statements
}

/// Returns instructions which sets raises an error if
/// the value in the given register is not of the given type.
///
/// This clobbers r9.
pub fn assert_type(register: Register, type_: &UnaryType) -> Vec<Statement> {
    vec![
        Statement::Mov {
            dest: R9,
            src: Operand::Register(register),
        },
        Statement::And {
            dest: R9,
            src: Operand::Immediate(type_.mask() as i64),
        },
        Statement::Cmp {
            dest: R9,
            src: Operand::Immediate(type_.tag.0 as i64),
        },
        Statement::Jne {
            label: ERR_LABEL.to_string(),
        },
    ]
}

pub fn assert_int(register: Register) -> Vec<Statement> {
    assert_type(register, &INT_TYPE)
}

pub fn assert_char(register: Register) -> Vec<Statement> {
    assert_type(register, &CHAR_TYPE)
}

pub fn assert_box(register: Register) -> Vec<Statement> {
    assert_type(register, &BOX_TYPE)
}

pub fn assert_cons(register: Register) -> Vec<Statement> {
    assert_type(register, &CONS_TYPE)
}

pub fn assert_vector(register: Register) -> Vec<Statement> {
    assert_type(register, &VECTOR_TYPE)
}

pub fn assert_string(register: Register) -> Vec<Statement> {
    assert_type(register, &STRING_TYPE)
}

pub fn assert_natural_number(register: Register) -> Vec<Statement> {
    let mut statements = assert_int(register.clone());
    statements.push(Statement::Cmp {
        dest: Operand::Register(register),
        src: Operand::from(Value::Int(0)),
    });
    statements.push(Statement::Jl {
        label: ERR_LABEL.to_string(),
    });
    statements
}

fn assert_codepoint() -> Vec<Statement> {
    let mut statements = assert_int(Register::RAX);

    // Make sure the value is in the range 0..=0x10FFFF
    statements.push(Statement::Cmp {
        dest: RAX,
        src: Operand::from(Value::Int(0)),
    });
    statements.push(Statement::Jl {
        label: ERR_LABEL.to_string(),
    });
    statements.push(Statement::Cmp {
        dest: RAX,
        src: Operand::from(Value::Int(0x10FFFF)),
    });
    statements.push(Statement::Jg {
        label: ERR_LABEL.to_string(),
    });

    // except for the range 55296..=57343.
    statements.push(Statement::Cmp {
        dest: RAX,
        src: Operand::from(Value::Int(55295)),
    });
    statements.push(Statement::Jl {
        label: "ok".to_string(),
    });
    statements.push(Statement::Cmp {
        dest: RAX,
        src: Operand::from(Value::Int(57344)),
    });
    statements.push(Statement::Jg {
        label: "ok".to_string(),
    });
    statements.push(Statement::Jmp {
        label: ERR_LABEL.to_string(),
    });
    statements.push(Statement::Label {
        name: "ok".to_string(),
    });
    statements
}

pub fn assert_byte(register: Register) -> Vec<Statement> {
    let mut statements = assert_int(register);

    // Make sure the value is in the range 0..=255
    statements.push(Statement::Cmp {
        dest: R9,
        src: Operand::from(Value::Int(0)),
    });
    statements.push(Statement::Jl {
        label: ERR_LABEL.to_string(),
    });
    statements.push(Statement::Cmp {
        dest: R9,
        src: Operand::from(Value::Int(255)),
    });
    statements.push(Statement::Jg {
        label: ERR_LABEL.to_string(),
    });

    statements
}
