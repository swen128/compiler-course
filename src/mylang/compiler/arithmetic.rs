use crate::{
    a86::ast::{Operand, Register, Statement},
    mylang::data_type::Value,
};

use super::{
    helper::{if_equal, if_less_than},
    types::assert_int,
};

const RAX: Operand = Operand::Register(Register::RAX);
const R8: Operand = Operand::Register(Register::R8);

/// Returns instructions which adds two integers in rax and r8.
pub fn compile_add() -> Vec<Statement> {
    let mut statements = assert_int(Register::RAX);
    statements.extend(assert_int(Register::R8));
    statements.push(Statement::Add { dest: RAX, src: R8 });
    statements
}

/// Returns instructions which returns integer value of `r8 - rax`.
pub fn compile_sub() -> Vec<Statement> {
    let mut statements = assert_int(Register::RAX);
    statements.extend(assert_int(Register::R8));
    statements.push(Statement::Sub { dest: R8, src: RAX });
    statements.push(Statement::Mov { dest: RAX, src: R8 });
    statements
}

/// Returns instructions which adds 1 to an integer in rax.
pub fn compile_add1() -> Vec<Statement> {
    let mut statements = assert_int(Register::RAX);
    statements.push(Statement::Add {
        dest: RAX,
        src: Operand::from(Value::Int(1)),
    });
    statements
}

/// Returns instructions which subtracts 1 from an integer in rax.
pub fn compile_sub1() -> Vec<Statement> {
    let mut statements = assert_int(Register::RAX);
    statements.push(Statement::Sub {
        dest: RAX,
        src: Operand::from(Value::Int(1)),
    });
    statements
}

/// Returns instructions which sets rax to true if rax is zero.
/// It raises an error if rax is not integer.
pub fn compile_is_zero() -> Vec<Statement> {
    let mut statements = assert_int(Register::RAX);
    statements.push(Statement::Cmp {
        dest: RAX,
        src: Operand::from(Value::Int(0)),
    });
    statements.extend(if_equal());
    statements
}

/// Returns instructions which sets rax to true if rax and r8 are equal integers.
/// It raises an error if rax or r8 is not integer.
pub fn compile_int_equal() -> Vec<Statement> {
    let mut statements = assert_int(Register::RAX);
    statements.extend(assert_int(Register::R8));
    statements.push(Statement::Cmp { dest: RAX, src: R8 });
    statements.extend(if_equal());
    statements
}

/// Returns instructions which sets rax to true if rax is less than r8.
/// It raises an error if rax or r8 is not integer.
pub fn compile_less_than() -> Vec<Statement> {
    let mut statements = assert_int(Register::RAX);
    statements.extend(assert_int(Register::R8));
    statements.push(Statement::Cmp { dest: RAX, src: R8 });
    statements.extend(if_less_than());
    statements
}
