use super::ast;
use crate::a86::ast::*;

struct Compiler {
    last_label_id: usize,
}

impl Compiler {
    fn new() -> Compiler {
        Compiler { last_label_id: 0 }
    }

    fn new_label_id(&mut self) -> String {
        self.last_label_id += 1;
        self.last_label_id.to_string()
    }
}

pub fn compile(program: ast::Program) -> Program {
    let mut compiler = Compiler::new();
    let mut statements = vec![
        Statement::Global {
            name: "entry".to_string(),
        },
        Statement::Label {
            name: "entry".to_string(),
        },
    ];
    statements.extend(compile_expr(program.expr, &mut compiler));
    statements.push(Statement::Ret);
    Program { statements }
}

fn compile_expr(expr: ast::Expr, compiler: &mut Compiler) -> Vec<Statement> {
    match expr {
        ast::Expr::Lit(lit) => compile_literal(lit),

        ast::Expr::Prim1(op, expr) => match op {
            ast::Op1::Add1 => compile_add1(*expr, compiler),
            ast::Op1::Sub1 => compile_sub1(*expr, compiler),
        },

        ast::Expr::IfZero(if_zero) => compile_if_zero(if_zero, compiler),
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

fn compile_add1(child: ast::Expr, compiler: &mut Compiler) -> Vec<Statement> {
    let mut statements = compile_expr(child, compiler);
    statements.push(Statement::Add {
        dest: Operand::Register(Register::RAX),
        src: Operand::Immediate(1),
    });
    statements
}

fn compile_sub1(child: ast::Expr, compiler: &mut Compiler) -> Vec<Statement> {
    let mut statements = compile_expr(child, compiler);
    statements.push(Statement::Sub {
        dest: Operand::Register(Register::RAX),
        src: Operand::Immediate(1),
    });
    statements
}

fn compile_if_zero(if_zero: ast::IfZero, compiler: &mut Compiler) -> Vec<Statement> {
    let label_id = compiler.new_label_id();
    let then_label = format!("then_{}", label_id);
    let end_label = format!("end_{}", label_id);

    let mut statements = compile_expr(*if_zero.cond, compiler);
    statements.push(Statement::Cmp {
        dest: Operand::Register(Register::RAX),
        src: Operand::Immediate(0),
    });
    statements.push(Statement::Je {
        label: then_label.clone(),
    });
    statements.extend(compile_expr(*if_zero.els, compiler));
    statements.push(Statement::Jmp {
        label: end_label.clone(),
    });
    statements.push(Statement::Label {
        name: then_label.clone(),
    });
    statements.extend(compile_expr(*if_zero.then, compiler));
    statements.push(Statement::Label {
        name: end_label.clone(),
    });
    statements
}
