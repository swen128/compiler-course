use super::ast;
use super::data_type::{Value, CHAR_SHIFT, INT_SHIFT, MASK_CHAR, TYPE_CHAR};
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
        Statement::Extern {
            name: "read_byte".to_string(),
        },
        Statement::Extern {
            name: "peek_byte".to_string(),
        },
        Statement::Extern {
            name: "write_byte".to_string(),
        },
        Statement::Label {
            name: "entry".to_string(),
        },
        Statement::Sub {
            dest: Operand::Register(Register::RSP),
            src: Operand::Immediate(8),
        },
    ];
    statements.extend(compile_expr(program.expr, &mut compiler));
    statements.push(Statement::Add {
        dest: Operand::Register(Register::RSP),
        src: Operand::Immediate(8),
    });
    statements.push(Statement::Ret);
    Program { statements }
}

fn compile_expr(expr: ast::Expr, compiler: &mut Compiler) -> Vec<Statement> {
    match expr {
        ast::Expr::Eof => compile_eof(),

        ast::Expr::Lit(lit) => compile_literal(lit),

        ast::Expr::Prim0(op) => match op {
            ast::Op0::ReadByte => compile_read_byte(),
            ast::Op0::PeekByte => compile_peek_byte(),
        },

        ast::Expr::Prim1(op, expr) => compile_prim1(op, *expr, compiler),

        ast::Expr::Begin(first, second) => compile_begin(*first, *second, compiler),

        ast::Expr::If(if_zero) => compile_if_expr(if_zero, compiler),
    }
}

fn compile_literal(lit: ast::Lit) -> Vec<Statement> {
    match lit {
        ast::Lit::Int(i) => compile_value(Value::Int(i)),
        ast::Lit::Bool(b) => compile_value(Value::Boolean(b)),
        ast::Lit::Char(c) => compile_value(Value::Char(c)),
    }
}

fn compile_eof() -> Vec<Statement> {
    compile_value(Value::Eof)
}

fn compile_value(value: Value) -> Vec<Statement> {
    vec![Statement::Mov {
        dest: Operand::Register(Register::RAX),
        src: Operand::Immediate(value.encode()),
    }]
}

fn compile_read_byte() -> Vec<Statement> {
    vec![Statement::Call {
        label: "read_byte".to_string(),
    }]
}

fn compile_peek_byte() -> Vec<Statement> {
    vec![Statement::Call {
        label: "peek_byte".to_string(),
    }]
}

fn compile_prim1(op: ast::Op1, expr: ast::Expr, compiler: &mut Compiler) -> Vec<Statement> {
    let mut statements = compile_expr(expr, compiler);
    statements.extend(compile_op1(op));
    statements
}

fn compile_op1(op: ast::Op1) -> Vec<Statement> {
    match op {
        ast::Op1::Add1 => vec![Statement::Add {
            dest: Operand::Register(Register::RAX),
            src: Operand::Immediate(Value::Int(1).encode()),
        }],

        ast::Op1::Sub1 => vec![Statement::Sub {
            dest: Operand::Register(Register::RAX),
            src: Operand::Immediate(Value::Int(1).encode()),
        }],

        ast::Op1::IsZero => {
            let mut statements = vec![Statement::Cmp {
                dest: Operand::Register(Register::RAX),
                src: Operand::Immediate(Value::Int(0).encode()),
            }];
            statements.extend(if_equal());
            statements
        }

        ast::Op1::IsChar => {
            let mut statements = vec![
                Statement::And {
                    dest: Operand::Register(Register::RAX),
                    src: Operand::Immediate(MASK_CHAR),
                },
                Statement::Cmp {
                    dest: Operand::Register(Register::RAX),
                    src: Operand::Immediate(TYPE_CHAR),
                },
            ];
            statements.extend(if_equal());
            statements
        }

        ast::Op1::IsEof => {
            let mut statements = vec![Statement::Cmp {
                dest: Operand::Register(Register::RAX),
                src: Operand::Immediate(Value::Eof.encode()),
            }];
            statements.extend(if_equal());
            statements
        }

        ast::Op1::CharToInt => vec![
            Statement::Sar {
                dest: Operand::Register(Register::RAX),
                src: Operand::Immediate(CHAR_SHIFT),
            },
            Statement::Sal {
                dest: Operand::Register(Register::RAX),
                src: Operand::Immediate(INT_SHIFT),
            },
        ],

        ast::Op1::IntToChar => vec![
            Statement::Sar {
                dest: Operand::Register(Register::RAX),
                src: Operand::Immediate(INT_SHIFT),
            },
            Statement::Sal {
                dest: Operand::Register(Register::RAX),
                src: Operand::Immediate(CHAR_SHIFT),
            },
            Statement::Xor {
                dest: Operand::Register(Register::RAX),
                src: Operand::Immediate(TYPE_CHAR),
            },
        ],

        ast::Op1::WriteByte => vec![
            Statement::Mov {
                dest: Operand::Register(Register::RDI),
                src: Operand::Register(Register::RAX),
            },
            Statement::Call {
                label: "write_byte".to_string(),
            },
        ],
    }
}

/// Set rax to true if the comparison flag is equal.
fn if_equal() -> Vec<Statement> {
    vec![
        Statement::Mov {
            dest: Operand::Register(Register::RAX),
            src: Operand::Immediate(Value::Boolean(false).encode()),
        },
        Statement::Mov {
            dest: Operand::Register(Register::R9),
            src: Operand::Immediate(Value::Boolean(true).encode()),
        },
        Statement::Cmove {
            dest: Operand::Register(Register::RAX),
            src: Operand::Register(Register::R9),
        },
    ]
}

fn compile_begin(first: ast::Expr, second: ast::Expr, compiler: &mut Compiler) -> Vec<Statement> {
    let mut statements = compile_expr(first, compiler);
    statements.extend(compile_expr(second, compiler));
    statements
}

fn compile_if_expr(if_expr: ast::If, compiler: &mut Compiler) -> Vec<Statement> {
    let label_id = compiler.new_label_id();
    let else_label = format!("else_{}", label_id);
    let end_label = format!("end_{}", label_id);

    let mut statements = compile_expr(*if_expr.cond, compiler);
    statements.push(Statement::Cmp {
        dest: Operand::Register(Register::RAX),
        src: Operand::Immediate(Value::Boolean(false).encode()),
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
