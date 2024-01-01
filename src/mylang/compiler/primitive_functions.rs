use crate::a86::ast::*;
use crate::mylang::ast;
use crate::mylang::data_type::{BOX_TYPE, CHAR_TYPE, CONS_TYPE, STRING_TYPE, VECTOR_TYPE};

use super::arithmetic::*;
use super::box_type::*;
use super::cons::*;
use super::expr::compile_expr;
use super::external_call::*;
use super::state::Compiler;
use super::string::*;
use super::types::*;
use super::vector::*;

pub fn compile_prim0(op: ast::Op0) -> Vec<Statement> {
    match op {
        ast::Op0::ReadByte => compile_read_byte(),
        ast::Op0::PeekByte => compile_peek_byte(),
    }
}

pub fn compile_prim1(op: ast::Op1, expr: ast::Expr, compiler: &mut Compiler) -> Vec<Statement> {
    let mut statements = compile_expr(expr, compiler);
    statements.extend(compile_op1(op));
    statements
}

pub fn compile_prim2(
    op: ast::Op2,
    first: ast::Expr,
    second: ast::Expr,
    compiler: &mut Compiler,
) -> Vec<Statement> {
    let mut statements = compile_expr(first, compiler);
    statements.push(Statement::Push {
        src: Operand::Register(Register::RAX),
    });
    compiler.variables_table.push_non_variable();
    statements.extend(compile_expr(second, compiler));
    statements.push(Statement::Pop {
        dest: Operand::Register(Register::R8),
    });
    compiler.variables_table.pop();
    statements.extend(compile_op2(op, compiler));
    statements
}

pub fn compile_prim3(
    op: ast::Op3,
    first: ast::Expr,
    second: ast::Expr,
    third: ast::Expr,
    compiler: &mut Compiler,
) -> Vec<Statement> {
    let mut statements = compile_expr(first, compiler);
    statements.push(Statement::Push {
        src: Operand::Register(Register::RAX),
    });
    compiler.variables_table.push_non_variable();
    statements.extend(compile_expr(second, compiler));
    statements.push(Statement::Push {
        src: Operand::Register(Register::RAX),
    });
    compiler.variables_table.push_non_variable();
    statements.extend(compile_expr(third, compiler));
    statements.extend(compile_op3(op, compiler));
    statements
}

/// Returns instructions which apply the given unary operator to the value in rax.
fn compile_op1(op: ast::Op1) -> Vec<Statement> {
    match op {
        ast::Op1::Add1 => compile_add1(),
        ast::Op1::Sub1 => compile_sub1(),

        ast::Op1::IsZero => compile_is_zero(),

        ast::Op1::IsChar => is_type(&CHAR_TYPE),
        ast::Op1::IsBox => is_type(&BOX_TYPE),
        ast::Op1::IsCons => is_type(&CONS_TYPE),
        ast::Op1::IsVector => is_type(&VECTOR_TYPE),
        ast::Op1::IsString => is_type(&STRING_TYPE),

        ast::Op1::IsEof => is_eof(),

        ast::Op1::CharToInt => char_to_int(),
        ast::Op1::IntToChar => int_to_char(),

        ast::Op1::WriteByte => compile_write_byte(),

        ast::Op1::Box => compile_box(),
        ast::Op1::Unbox => compile_unbox(),

        ast::Op1::Car => compile_car(),
        ast::Op1::Cdr => compile_cdr(),
    }
}

/// Returns instructions which apply the given binary operator to the values in
/// r8 (first operand) and rax (second operand).
fn compile_op2(op: ast::Op2, compiler: &mut Compiler) -> Vec<Statement> {
    match op {
        ast::Op2::Add => compile_add(),
        ast::Op2::Sub => compile_sub(),
        ast::Op2::Equal => compile_int_equal(),
        ast::Op2::LessThan => compile_less_than(),

        ast::Op2::Cons => compile_cons(),

        ast::Op2::MakeVector => compile_make_vector(compiler),
        ast::Op2::VectorRef => compile_vector_ref(compiler),

        ast::Op2::MakeString => compile_make_string(compiler),
        ast::Op2::StringRef => compile_string_ref(compiler),
    }
}

/// Returns instructions which apply the given ternary operator to the arguments,
/// which are assumed to be laid out as follows:
/// * 1st argument: Second of the stack
/// * 2nd argument: Topmost of the stack
/// * 3rd argument: rax
fn compile_op3(op: ast::Op3, compiler: &mut Compiler) -> Vec<Statement> {
    match op {
        ast::Op3::VectorSet => compile_vector_set(compiler),
    }
}
