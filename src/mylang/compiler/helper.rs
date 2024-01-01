use crate::{
    a86::ast::{Operand, Register, Statement},
    mylang::data_type::Value,
};

const RAX: Operand = Operand::Register(Register::RAX);
const R9: Operand = Operand::Register(Register::R9);

/// Returns instructions which sets rax to true if the comparison flag is equal.
/// This clobbers r9.
pub fn if_equal() -> Vec<Statement> {
    vec![
        Statement::Mov {
            dest: RAX,
            src: Operand::Immediate(Value::Boolean(false).encode()),
        },
        Statement::Mov {
            dest: R9,
            src: Operand::Immediate(Value::Boolean(true).encode()),
        },
        Statement::Cmove { dest: RAX, src: R9 },
    ]
}

/// Returns instructions which sets rax to true if the comparison flag is less.
/// This clobbers r9.
pub fn if_less_than() -> Vec<Statement> {
    vec![
        Statement::Mov {
            dest: RAX,
            src: Operand::Immediate(Value::Boolean(false).encode()),
        },
        Statement::Mov {
            dest: R9,
            src: Operand::Immediate(Value::Boolean(true).encode()),
        },
        Statement::Cmovl { dest: RAX, src: R9 },
    ]
}
