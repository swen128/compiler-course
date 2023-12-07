use crate::a86::ast::*;
use super::ast;

pub fn compile(program: ast::Program) -> Program {
    let mut statements = vec![
        Statement::Global {
            name: "entry".to_string(),
        },
        Statement::Label {
            name: "entry".to_string(),
        },
    ];
    statements.extend(compile_expr(program.expr));
    statements.push(Statement::Ret);
    Program { statements }
}

fn compile_expr(expr: ast::Expr) -> Vec<Statement> {
    match expr {
        ast::Expr::Lit(lit) => compile_literal(lit),

        ast::Expr::Prim1(op, expr) => match op {
            ast::Op1::Add1 => compile_add1(*expr),
            ast::Op1::Sub1 => compile_sub1(*expr),
        },
    }
}

fn compile_literal(lit: ast::Lit) -> Vec<Statement> {
    match lit {
        ast::Lit::Int(i) => {
            vec![Statement::Mov {
                dest: Operand::Register(Register::RAX),
                src: Operand::Immediate(i),
            }]
        }
    }
}

fn compile_add1(child: ast::Expr) -> Vec<Statement> {
    let mut statements = compile_expr(child);
    statements.push(Statement::Add {
        dest: Operand::Register(Register::RAX),
        src: Operand::Immediate(1),
    });
    statements
}

fn compile_sub1(child: ast::Expr) -> Vec<Statement> {
    let mut statements = compile_expr(child);
    statements.push(Statement::Sub {
        dest: Operand::Register(Register::RAX),
        src: Operand::Immediate(1),
    });
    statements
}
