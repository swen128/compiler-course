use crate::a86::ast::*;
use crate::mylang::ast;
use crate::mylang::data_type::Value;

use super::function::compile_function_application;
use super::primitive_functions::{compile_prim0, compile_prim1, compile_prim2, compile_prim3};
use super::state::Compiler;
use super::string::compile_string_literal;
use super::variable::{compile_let, compile_variable, VariablesTable};

const RAX: Operand = Operand::Register(Register::RAX);

pub fn compile_expr(
    expr: ast::Expr,
    compiler: &mut Compiler,
    env: &VariablesTable,
) -> Vec<Statement> {
    match expr {
        ast::Expr::Eof => compile_value(Value::Eof),
        ast::Expr::Lit(lit) => compile_value(Value::from(lit)),
        ast::Expr::String(string) => compile_string_literal(&string),

        ast::Expr::Prim0(op) => compile_prim0(op),
        ast::Expr::Prim1(op, expr) => compile_prim1(op, *expr, compiler, env),
        ast::Expr::Prim2(op, first, second) => compile_prim2(op, *first, *second, compiler, env),
        ast::Expr::Prim3(op, first, second, third) => {
            compile_prim3(op, *first, *second, *third, compiler, env)
        }

        ast::Expr::Begin(first, second) => compile_begin(*first, *second, compiler, env),

        ast::Expr::If(if_zero) => compile_if_expr(if_zero, compiler, env),

        ast::Expr::Variable(variable) => compile_variable(variable, compiler, env),
        ast::Expr::Let(let_expr) => compile_let(let_expr, compiler, env),

        ast::Expr::App(app) => compile_function_application(app, compiler, env),
    }
}

fn compile_value(value: Value) -> Vec<Statement> {
    vec![Statement::Mov {
        dest: RAX,
        src: Operand::from(value),
    }]
}

fn compile_begin(
    first: ast::Expr,
    second: ast::Expr,
    compiler: &mut Compiler,
    env: &VariablesTable,
) -> Vec<Statement> {
    let mut statements = compile_expr(first, compiler, env);
    statements.extend(compile_expr(second, compiler, env));
    statements
}

fn compile_if_expr(
    if_expr: ast::If,
    compiler: &mut Compiler,
    env: &VariablesTable,
) -> Vec<Statement> {
    let label_id = compiler.new_label_id();
    let else_label = format!("else_{}", label_id);
    let end_label = format!("end_{}", label_id);

    let mut statements = compile_expr(*if_expr.cond, compiler, env);
    statements.push(Statement::Cmp {
        dest: RAX,
        src: Operand::from(Value::Boolean(false)),
    });
    statements.push(Statement::Je {
        label: else_label.clone(),
    });
    statements.extend(compile_expr(*if_expr.then, compiler, env));
    statements.push(Statement::Jmp {
        label: end_label.clone(),
    });
    statements.push(Statement::Label {
        name: else_label.clone(),
    });
    statements.extend(compile_expr(*if_expr.els, compiler, env));
    statements.push(Statement::Label {
        name: end_label.clone(),
    });
    statements
}

impl From<Value> for Operand {
    fn from(value: Value) -> Self {
        Operand::Immediate(value.encode())
    }
}
