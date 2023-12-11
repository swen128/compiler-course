use super::ast;
use super::data_type::Value;
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
            ast::Op1::IsZero => compile_is_zero(*expr, compiler),
        },

        ast::Expr::If(if_zero) => compile_if_expr(if_zero, compiler),
    }
}

fn compile_literal(lit: ast::Lit) -> Vec<Statement> {
    match lit {
        ast::Lit::Int(i) => {
            vec![Statement::Mov {
                dest: Operand::Register(Register::RAX),
                src: Operand::Immediate(value_to_bits(Value::Int(i))),
            }]
        }
        ast::Lit::Bool(b) => {
            vec![Statement::Mov {
                dest: Operand::Register(Register::RAX),
                src: Operand::Immediate(value_to_bits(Value::Boolean(b))),
            }]
        }
    }
}

fn compile_add1(child: ast::Expr, compiler: &mut Compiler) -> Vec<Statement> {
    let mut statements = compile_expr(child, compiler);
    statements.push(Statement::Add {
        dest: Operand::Register(Register::RAX),
        src: Operand::Immediate(value_to_bits(Value::Int(1))),
    });
    statements
}

fn compile_sub1(child: ast::Expr, compiler: &mut Compiler) -> Vec<Statement> {
    let mut statements = compile_expr(child, compiler);
    statements.push(Statement::Sub {
        dest: Operand::Register(Register::RAX),
        src: Operand::Immediate(value_to_bits(Value::Int(1))),
    });
    statements
}

fn compile_is_zero(child: ast::Expr, compiler: &mut Compiler) -> Vec<Statement> {
    let mut statements = compile_expr(child, compiler);
    statements.push(Statement::Cmp {
        dest: Operand::Register(Register::RAX),
        src: Operand::Immediate(value_to_bits(Value::Int(0))),
    });
    statements.push(Statement::Mov {
        dest: Operand::Register(Register::RAX),
        src: Operand::Immediate(value_to_bits(Value::Boolean(false))),
    });
    statements.push(Statement::Mov {
        dest: Operand::Register(Register::R9),
        src: Operand::Immediate(value_to_bits(Value::Boolean(true))),
    });
    statements.push(Statement::Cmove {
        dest: Operand::Register(Register::RAX),
        src: Operand::Register(Register::R9),
    });
    statements
}

fn compile_if_expr(if_expr: ast::If, compiler: &mut Compiler) -> Vec<Statement> {
    let label_id = compiler.new_label_id();
    let else_label = format!("else_{}", label_id);
    let end_label = format!("end_{}", label_id);

    let mut statements = compile_expr(*if_expr.cond, compiler);
    statements.push(Statement::Cmp {
        dest: Operand::Register(Register::RAX),
        src: Operand::Immediate(value_to_bits(Value::Boolean(false))),
    });
    statements.push(Statement::Je {
        label: else_label.clone(),
    });
    statements.extend(compile_expr(*if_expr.then, compiler));
    statements.push(Statement::Jmp {
        label: end_label.clone(),
    });
    statements.push(Statement::Label {
        name: else_label.clone(),
    });
    statements.extend(compile_expr(*if_expr.els, compiler));
    statements.push(Statement::Label {
        name: end_label.clone(),
    });
    statements
}

fn value_to_bits(value: Value) -> i64 {
    match value {
        Value::Int(i) => i << 1,
        Value::Boolean(b) => {
            if b {
                3
            } else {
                1
            }
        }
    }
}
