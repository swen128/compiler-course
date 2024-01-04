use crate::a86::ast::*;
use crate::mylang::ast;
use crate::mylang::data_type::Value;

use super::function::compile_function_application;
use super::pattern_match::compile_match;
use super::primitive_functions::{compile_prim0, compile_prim1, compile_prim2, compile_prim3};
use super::state::Compiler;
use super::string::compile_string_literal;
use super::variable::{compile_let, compile_variable, VariablesTable};

const RAX: Operand = Operand::Register(Register::RAX);

/// Returns instructions to set rax to the evaluated value of the given expression.
///
/// # Arguments
/// * `expr` - The expression to compile.
/// * `compiler` - The global compiler state.
/// * `env` - The variables in current scope.
/// * `is_tail_expr` - Whether the expression is in tail position.
pub fn compile_expr(
    expr: ast::Expr,
    compiler: &mut Compiler,
    env: &VariablesTable,
    is_tail_expr: bool,
) -> Vec<Statement> {
    match expr {
        ast::Expr::Eof => compile_value(Value::Eof),
        ast::Expr::Lit(lit) => compile_literal(lit),

        ast::Expr::Prim0(op) => compile_prim0(op),
        ast::Expr::Prim1(op, expr) => compile_prim1(op, *expr, compiler, env),
        ast::Expr::Prim2(op, first, second) => compile_prim2(op, *first, *second, compiler, env),
        ast::Expr::Prim3(op, first, second, third) => {
            compile_prim3(op, *first, *second, *third, compiler, env)
        }

        ast::Expr::Begin(first, second) => {
            compile_begin(*first, *second, compiler, env, is_tail_expr)
        }

        ast::Expr::If(if_zero) => compile_if_expr(if_zero, compiler, env, is_tail_expr),
        
        ast::Expr::Match(match_expr) => compile_match(match_expr, compiler, env, is_tail_expr),

        ast::Expr::Variable(variable) => compile_variable(variable, compiler, env),
        ast::Expr::Let(let_expr) => compile_let(let_expr, compiler, env, is_tail_expr),

        ast::Expr::App(app) => compile_function_application(app, compiler, env, is_tail_expr),
    }
}

fn compile_literal(lit: ast::Lit) -> Vec<Statement> {
    match lit {
        ast::Lit::Int(i) => compile_value(Value::Int(i)),
        ast::Lit::Bool(b) => compile_value(Value::Boolean(b)),
        ast::Lit::Char(c) => compile_value(Value::Char(c)),
        ast::Lit::String(s) => compile_string_literal(&s),
        ast::Lit::EmptyList => compile_value(Value::EmptyList),
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
    is_tail_expr: bool,
) -> Vec<Statement> {
    let mut statements = compile_expr(first, compiler, env, false);
    statements.extend(compile_expr(second, compiler, env, is_tail_expr));
    statements
}

fn compile_if_expr(
    if_expr: ast::If,
    compiler: &mut Compiler,
    env: &VariablesTable,
    is_tail_expr: bool,
) -> Vec<Statement> {
    let label_id = compiler.new_label_id();
    let else_label = format!("else_{}", label_id);
    let end_label = format!("end_{}", label_id);

    let mut statements = compile_expr(*if_expr.cond, compiler, env, false);
    statements.push(Statement::Cmp {
        dest: RAX,
        src: Operand::from(Value::Boolean(false)),
    });
    statements.push(Statement::Je {
        label: else_label.clone(),
    });
    statements.extend(compile_expr(*if_expr.then, compiler, env, is_tail_expr));
    statements.push(Statement::Jmp {
        label: end_label.clone(),
    });
    statements.push(Statement::Label {
        name: else_label.clone(),
    });
    statements.extend(compile_expr(*if_expr.els, compiler, env, is_tail_expr));
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
