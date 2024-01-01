use crate::{
    a86::ast::{Operand, Register, Statement},
    mylang::data_type::BOX_TYPE,
};

use super::types::assert_box;

const RAX: Operand = Operand::Register(Register::RAX);
const RBX: Operand = Operand::Register(Register::RBX);

pub fn compile_box() -> Vec<Statement> {
    vec![
        // Put the value into the heap.
        Statement::Mov {
            dest: Operand::Offset(Register::RBX, 0),
            src: RAX,
        },
        // Tag the address as box data type and return it.
        Statement::Mov {
            dest: RAX,
            src: RBX,
        },
        Statement::Or {
            dest: RAX,
            src: Operand::Immediate(BOX_TYPE.tag.0 as i64),
        },
        // Advance the heap pointer 1 word.
        Statement::Add {
            dest: RBX,
            src: Operand::Immediate(8),
        },
    ]
}

pub fn compile_unbox() -> Vec<Statement> {
    let mut statements = assert_box(Register::RAX);
    statements.push(Statement::Xor {
        dest: RAX,
        src: Operand::Immediate(BOX_TYPE.tag.0 as i64),
    });
    statements.push(Statement::Mov {
        dest: RAX,
        src: Operand::Offset(Register::RAX, 0),
    });
    statements
}
