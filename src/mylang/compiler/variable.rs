use crate::{
    a86::ast::{Operand, Register, Statement},
    mylang::ast::{self, Variable},
};

use super::{state::Compiler, expr::compile_expr};

const RAX: Operand = Operand::Register(Register::RAX);
const RSP: Operand = Operand::Register(Register::RSP);

pub fn compile_let(expr: ast::Let, compiler: &mut Compiler) -> Vec<Statement> {
    let ast::Let { binding, body } = expr;

    let mut statements = compile_expr(*binding.rhs, compiler);
    statements.push(Statement::Push { src: RAX });
    compiler.variables_table.push_variable(binding.lhs);
    statements.extend(compile_expr(*body, compiler));
    compiler.variables_table.pop();

    // Pop the value from the stack and discard it.
    statements.push(Statement::Add {
        dest: RSP,
        src: Operand::Immediate(8),
    });
    statements
}

pub fn compile_variable(variable: Variable, compiler: &mut Compiler) -> Vec<Statement> {
    let position = compiler
        .variables_table
        .position(&variable)
        .expect(format!("Undefined variable `{}`", variable.0).as_str()); // TODO: Return `Result` type.
    let offset = (position * 8) as i64;
    vec![Statement::Mov {
        dest: RAX,
        src: Operand::Offset(Register::RSP, offset),
    }]
}
