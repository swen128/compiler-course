use crate::a86::ast::{Operand, Register, Statement};

use super::types::assert_byte;

const RAX: Operand = Operand::Register(Register::RAX);
const RDI: Operand = Operand::Register(Register::RDI);
const R15: Operand = Operand::Register(Register::R15);
const RSP: Operand = Operand::Register(Register::RSP);

/// Return the instructions for declaring external functions.
/// Put the them at the beginning of the program.
pub fn externals() -> Vec<Statement> {
    vec![
        Statement::Extern {
            name: "read_byte".to_string(),
        },
        Statement::Extern {
            name: "peek_byte".to_string(),
        },
        Statement::Extern {
            name: "write_byte".to_string(),
        },
        Statement::Extern {
            name: "raise_error".to_string(),
        },
    ]
}

pub fn compile_read_byte() -> Vec<Statement> {
    call("read_byte".to_string())
}

pub fn compile_peek_byte() -> Vec<Statement> {
    call("peek_byte".to_string())
}

pub fn compile_write_byte() -> Vec<Statement> {
    let mut statements = assert_byte(Register::RAX);
    statements.push(Statement::Mov {
        dest: RDI,
        src: RAX,
    });
    statements.extend(call("write_byte".to_string()));
    statements
}

pub fn compile_raise_error() -> Vec<Statement> {
    let mut statements = pad_stack();
    statements.push(Statement::Call {
        label: "raise_error".to_string(),
    });
    statements
}

fn call(label: String) -> Vec<Statement> {
    let mut statements = pad_stack();
    statements.push(Statement::Call { label });
    statements.extend(unpad_stack());
    statements
}

/// Returns instructions which alligns the stack pointer to a 16-byte boundary.
/// This must be done before calling an external function.
/// After the call, the stack must be unaligned using [unpad_stack].
fn pad_stack() -> Vec<Statement> {
    vec![
        Statement::Mov {
            dest: R15,
            src: RSP,
        },
        Statement::And {
            dest: R15,
            src: Operand::Immediate(0b1000),
        },
        Statement::Sub {
            dest: RSP,
            src: R15,
        },
    ]
}

/// Returns instructions which undo the stack alignment done by [pad_stack].
fn unpad_stack() -> Vec<Statement> {
    vec![Statement::Add {
        dest: RSP,
        src: R15,
    }]
}
