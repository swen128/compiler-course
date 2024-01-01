use crate::{
    a86::ast::{Operand, Register, Statement},
    mylang::data_type::CONS_TYPE,
};

use super::types::assert_cons;

const RAX: Operand = Operand::Register(Register::RAX);
const R8: Operand = Operand::Register(Register::R8);
const RBX: Operand = Operand::Register(Register::RBX);

/// Returns instructions which creates a cons cell,
/// assuming the first value is in rax and the second value is in r8.
pub fn compile_cons() -> Vec<Statement> {
    vec![
        // Put the values into the heap.
        Statement::Mov {
            dest: Operand::Offset(Register::RBX, 0),
            src: RAX,
        },
        Statement::Mov {
            dest: Operand::Offset(Register::RBX, 8),
            src: R8,
        },
        // Tag the address as cons data type and return it.
        Statement::Mov {
            dest: RAX,
            src: RBX,
        },
        Statement::Or {
            dest: RAX,
            src: Operand::Immediate(CONS_TYPE.tag.0 as i64),
        },
        // Advance the heap pointer 2 words.
        Statement::Add {
            dest: RBX,
            src: Operand::Immediate(16),
        },
    ]
}

pub fn compile_car() -> Vec<Statement> {
    let mut statements = assert_cons(Register::RAX);
    statements.push(Statement::Xor {
        dest: RAX,
        src: Operand::Immediate(CONS_TYPE.tag.0 as i64),
    });
    statements.push(Statement::Mov {
        dest: RAX,
        src: Operand::Offset(Register::RAX, 8),
    });
    statements
}

pub fn compile_cdr() -> Vec<Statement> {
    let mut statements = assert_cons(Register::RAX);
    statements.push(Statement::Xor {
        dest: RAX,
        src: Operand::Immediate(CONS_TYPE.tag.0 as i64),
    });
    statements.push(Statement::Mov {
        dest: RAX,
        src: Operand::Offset(Register::RAX, 0),
    });
    statements
}
