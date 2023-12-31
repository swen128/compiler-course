use super::ast::{self, Variable};
use super::data_type::{
    UnaryType, Value, BOX_TYPE, CHAR_TYPE, CONS_TYPE, INT_TYPE, STRING_TYPE, VECTOR_TYPE,
};
use crate::a86::ast::*;

const RAX: Operand = Operand::Register(Register::RAX);
const EAX: Operand = Operand::Register(Register::EAX);
const RBX: Operand = Operand::Register(Register::RBX);
const RSP: Operand = Operand::Register(Register::RSP);
const RDI: Operand = Operand::Register(Register::RDI);
const R8: Operand = Operand::Register(Register::R8);
const R9: Operand = Operand::Register(Register::R9);
const R10: Operand = Operand::Register(Register::R10);
const R15: Operand = Operand::Register(Register::R15);

const ERR_LABEL: &str = "err";

struct Compiler {
    last_label_id: usize,
    variables_table: VariablesTable,
}

impl Compiler {
    fn new() -> Compiler {
        Compiler {
            last_label_id: 0,
            variables_table: VariablesTable::new(),
        }
    }

    fn new_label_id(&mut self) -> String {
        self.last_label_id += 1;
        self.last_label_id.to_string()
    }
}

struct VariablesTable {
    variables: Vec<Option<Variable>>,
}

/// Keeps track of local variables, mapping them to lexical addresses.
impl VariablesTable {
    fn new() -> Self {
        Self {
            variables: Vec::new(),
        }
    }

    /// Pushes a new variable to the stack.
    /// This should be called when binding a new variable in the `let` expression.
    fn push_variable(&mut self, variable: Variable) {
        self.variables.push(Some(variable));
    }

    /// Pushes a new non-variable to the stack.
    /// This should be called whenever pushing an arbitrary non-variable value to the stack.
    fn push_non_variable(&mut self) {
        self.variables.push(None);
    }

    fn pop(&mut self) {
        self.variables.pop();
    }

    fn position(&self, variable: &Variable) -> Option<usize> {
        self.variables
            .iter()
            .position(|option| option.as_ref().is_some_and(|v| v == variable))
            .map(|i| self.variables.len() - i - 1)
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
        Statement::Extern {
            name: "raise_error".to_string(),
        },
        Statement::Label {
            name: "entry".to_string(),
        },
        Statement::Mov {
            dest: RBX,
            src: RDI, // The runtime must allocate the heap memory and pass its address via rdi.
        },
    ];
    statements.extend(compile_expr(program.expr, &mut compiler));
    statements.push(Statement::Ret);
    statements.push(Statement::Label {
        name: ERR_LABEL.to_string(),
    });
    statements.extend(pad_stack());
    statements.push(Statement::Call {
        label: "raise_error".to_string(),
    });
    Program { statements }
}

fn compile_expr(expr: ast::Expr, compiler: &mut Compiler) -> Vec<Statement> {
    match expr {
        ast::Expr::Eof => compile_eof(),

        ast::Expr::Lit(lit) => compile_literal(lit),

        ast::Expr::String(string) => compile_string_literal(&string),

        ast::Expr::Prim0(op) => match op {
            ast::Op0::ReadByte => compile_read_byte(),
            ast::Op0::PeekByte => compile_peek_byte(),
        },

        ast::Expr::Prim1(op, expr) => compile_prim1(op, *expr, compiler),
        ast::Expr::Prim2(op, first, second) => compile_prim2(op, *first, *second, compiler),
        ast::Expr::Prim3(op, first, second, third) => {
            compile_prim3(op, *first, *second, *third, compiler)
        }

        ast::Expr::Begin(first, second) => compile_begin(*first, *second, compiler),

        ast::Expr::If(if_zero) => compile_if_expr(if_zero, compiler),

        ast::Expr::Variable(variable) => compile_variable(variable, compiler),

        ast::Expr::Let(let_expr) => compile_let(let_expr, compiler),
    }
}

fn compile_literal(lit: ast::Lit) -> Vec<Statement> {
    compile_value(Value::from(lit))
}

fn compile_eof() -> Vec<Statement> {
    compile_value(Value::Eof)
}

fn compile_value(value: Value) -> Vec<Statement> {
    vec![Statement::Mov {
        dest: RAX,
        src: Operand::from(value),
    }]
}

fn compile_read_byte() -> Vec<Statement> {
    call("read_byte".to_string())
}

fn compile_peek_byte() -> Vec<Statement> {
    call("peek_byte".to_string())
}

fn compile_prim1(op: ast::Op1, expr: ast::Expr, compiler: &mut Compiler) -> Vec<Statement> {
    let mut statements = compile_expr(expr, compiler);
    statements.extend(compile_op1(op));
    statements
}

fn compile_prim2(
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

fn compile_prim3(
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
        ast::Op1::Add1 => {
            let mut statements = assert_int(Register::RAX);
            statements.push(Statement::Add {
                dest: RAX,
                src: Operand::from(Value::Int(1)),
            });
            statements
        }

        ast::Op1::Sub1 => {
            let mut statements = assert_int(Register::RAX);
            statements.push(Statement::Sub {
                dest: RAX,
                src: Operand::from(Value::Int(1)),
            });
            statements
        }

        ast::Op1::IsZero => {
            let mut statements = assert_int(Register::RAX);
            statements.push(Statement::Cmp {
                dest: RAX,
                src: Operand::Immediate(Value::Int(0).encode()),
            });
            statements.extend(if_equal());
            statements
        }

        ast::Op1::IsChar => is_type(&CHAR_TYPE),
        ast::Op1::IsBox => is_type(&BOX_TYPE),
        ast::Op1::IsCons => is_type(&CONS_TYPE),
        ast::Op1::IsVector => is_type(&VECTOR_TYPE),
        ast::Op1::IsString => is_type(&STRING_TYPE),

        ast::Op1::IsEof => {
            let mut statements = vec![Statement::Cmp {
                dest: RAX,
                src: Operand::Immediate(Value::Eof.encode()),
            }];
            statements.extend(if_equal());
            statements
        }

        ast::Op1::CharToInt => {
            let mut statements = assert_char(Register::RAX);
            statements.extend(cast_type(Register::RAX, &CHAR_TYPE, &INT_TYPE));
            statements
        }

        ast::Op1::IntToChar => {
            let mut statements = assert_codepoint();
            statements.extend(cast_type(Register::RAX, &INT_TYPE, &CHAR_TYPE));
            statements
        }

        ast::Op1::WriteByte => {
            let mut statements = assert_byte(Register::RAX);
            statements.push(Statement::Mov {
                dest: RDI,
                src: RAX,
            });
            statements.extend(call("write_byte".to_string()));
            statements
        }

        ast::Op1::Box => {
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

        ast::Op1::Unbox => {
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

        ast::Op1::Car => {
            let mut statements = assert_cons(Register::RAX);
            statements.push(Statement::Xor {
                dest: RAX,
                src: Operand::Immediate(CONS_TYPE.tag.0 as i64),
            });
            statements.push(Statement::Mov {
                dest: RAX,
                src: Operand::Offset(Register::RAX, 8),
            });
            statements
        }

        ast::Op1::Cdr => {
            let mut statements = assert_cons(Register::RAX);
            statements.push(Statement::Xor {
                dest: RAX,
                src: Operand::Immediate(CONS_TYPE.tag.0 as i64),
            });
            statements.push(Statement::Mov {
                dest: RAX,
                src: Operand::Offset(Register::RAX, 0),
            });
            statements
        }
    }
}

/// Returns instructions which apply the given binary operator to the values in
/// r8 (first operand) and rax (second operand).
fn compile_op2(op: ast::Op2, compiler: &mut Compiler) -> Vec<Statement> {
    match op {
        ast::Op2::Add => {
            let mut statements = assert_int(Register::RAX);
            statements.extend(assert_int(Register::R8));
            statements.push(Statement::Add { dest: RAX, src: R8 });
            statements
        }

        ast::Op2::Sub => {
            let mut statements = assert_int(Register::RAX);
            statements.extend(assert_int(Register::R8));
            statements.push(Statement::Sub { dest: R8, src: RAX });
            statements.push(Statement::Mov { dest: RAX, src: R8 });
            statements
        }

        ast::Op2::Equal => {
            let mut statements = assert_int(Register::RAX);
            statements.extend(assert_int(Register::R8));
            statements.push(Statement::Cmp { dest: RAX, src: R8 });
            statements.extend(if_equal());
            statements
        }

        ast::Op2::LessThan => {
            let mut statements = assert_int(Register::RAX);
            statements.extend(assert_int(Register::R8));
            statements.push(Statement::Cmp { dest: RAX, src: R8 });
            statements.extend(if_less_than());
            statements
        }

        ast::Op2::Cons => {
            vec![
                // Put the values into the heap.
                Statement::Mov {
                    dest: Operand::Offset(Register::RBX, 0),
                    src: RAX,
                },
                Statement::Mov {
                    dest: Operand::Offset(Register::RBX, 8),
                    src: R8,
                },
                // Tag the address as cons data type and return it.
                Statement::Mov {
                    dest: RAX,
                    src: RBX,
                },
                Statement::Or {
                    dest: RAX,
                    src: Operand::Immediate(CONS_TYPE.tag.0 as i64),
                },
                // Advance the heap pointer 2 words.
                Statement::Add {
                    dest: RBX,
                    src: Operand::Immediate(16),
                },
            ]
        }

        ast::Op2::MakeVector => compile_make_vector(compiler),
        ast::Op2::MakeString => compile_make_string(compiler),
        ast::Op2::VectorRef => compile_vector_ref(compiler),
        ast::Op2::StringRef => compile_string_ref(compiler),
    }
}

/// Returns instructions which apply the given ternary operator to the arguments,
/// which are assumed to be laid out as follows:
/// * 3rd argument: rax
/// * 2nd argument: Topmost of the stack
/// * 1st argument: Second of the stack
fn compile_op3(op: ast::Op3, compiler: &mut Compiler) -> Vec<Statement> {
    match op {
        ast::Op3::VectorSet => {
            // 1st argument (vector): r8
            // 2nd argument (index): r10
            // 3rd argument (new value): rax
            let mut statements = vec![
                Statement::Pop {
                    dest: Operand::Register(Register::R10),
                },
                Statement::Pop {
                    dest: Operand::Register(Register::R8),
                },
            ];
            compiler.variables_table.pop();
            compiler.variables_table.pop();
            statements.extend(assert_vector(Register::R8));
            statements.extend(assert_natural_number(Register::R10));

            // Cast r8 to the raw pointer address of the vector.
            statements.push(Statement::Xor {
                dest: R8,
                src: Operand::Immediate(VECTOR_TYPE.tag.0 as i64),
            });
            // Set r9 to the length of the vector.
            statements.push(Statement::Mov {
                dest: R9,
                src: Operand::Offset(Register::R8, 0),
            });
            // Cast r10 to raw integer representing the index.
            statements.push(Statement::Sar {
                dest: R10,
                src: Operand::Immediate(INT_TYPE.shift as i64),
            });

            // Check if the index is out of bounds. (length - 1 < index)
            statements.push(Statement::Sub {
                dest: R9,
                src: Operand::Immediate(1),
            });
            statements.push(Statement::Cmp { dest: R9, src: R10 });
            statements.push(Statement::Jl {
                label: ERR_LABEL.to_string(),
            });

            // Set the new value.
            statements.push(Statement::Sal {
                dest: R10,
                src: Operand::Immediate(3), // Multiply by 2^3 = 8 = 1 word
            });
            statements.push(Statement::Add { dest: R8, src: R10 });
            statements.push(Statement::Mov {
                dest: Operand::Offset(Register::R8, 8), // The offset 8 is required as the first word is the length.
                src: RAX,
            });

            // Return void.
            statements.push(Statement::Mov {
                dest: RAX,
                src: Operand::from(Value::Void),
            });
            statements
        }
    }
}

/// Returns instructions to initialize a vector of the given length with the repeated values,
/// assuming the length and the value is already given in r8 and rax respectively.
fn compile_make_vector(compiler: &mut Compiler) -> Vec<Statement> {
    let mut statements = assert_natural_number(Register::R8);

    let loop_label = format!("loop_{}", compiler.new_label_id());
    let end_label = format!("end_{}", compiler.new_label_id());
    let empty_label = format!("empty_{}", compiler.new_label_id());

    // Special case for empty vector
    statements.push(Statement::Cmp {
        dest: R8,
        src: Operand::from(Value::Int(0)),
    });
    statements.push(Statement::Je {
        label: empty_label.clone(),
    });

    // Stash the top address and cast it to the vector type.
    statements.push(Statement::Mov { dest: R9, src: RBX });
    statements.push(Statement::Or {
        dest: R9,
        src: Operand::Immediate(VECTOR_TYPE.tag.0 as i64),
    });

    // Put the length of the vector at the beginning.
    // Note: The length is guaranteed to be an integer, so we can strip the type tag.
    statements.push(Statement::Sar {
        dest: R8,
        src: Operand::Immediate(INT_TYPE.shift as i64),
    });
    statements.push(Statement::Mov {
        dest: Operand::Offset(Register::RBX, 0),
        src: R8,
    });
    statements.push(Statement::Add {
        dest: RBX,
        src: Operand::Immediate(8),
    });

    // Initialize each element of the vector to the given value.
    statements.push(Statement::Label {
        name: loop_label.clone(),
    });
    statements.push(Statement::Mov {
        dest: Operand::Offset(Register::RBX, 0),
        src: RAX,
    });
    statements.push(Statement::Add {
        dest: RBX,
        src: Operand::Immediate(8),
    });
    statements.push(Statement::Sub {
        dest: R8,
        src: Operand::Immediate(1),
    });
    statements.push(Statement::Cmp {
        dest: R8,
        src: Operand::Immediate(0),
    });
    statements.push(Statement::Jne {
        label: loop_label.clone(),
    });

    // Return the vector.
    statements.push(Statement::Mov { dest: RAX, src: R9 });
    statements.push(Statement::Jmp {
        label: end_label.clone(),
    });

    // Special case for empty vector
    statements.push(Statement::Label {
        name: empty_label.clone(),
    });
    statements.push(Statement::Mov {
        dest: RAX,
        src: Operand::from(Value::EmptyVector),
    });

    statements.push(Statement::Label {
        name: end_label.clone(),
    });
    statements
}

/// Returns instructions to initialize a string of the given length with the repeated values,
/// assuming the length and the value is already given in r8 and rax respectively.
fn compile_make_string(compiler: &mut Compiler) -> Vec<Statement> {
    let mut statements = assert_natural_number(Register::R8);
    statements.extend(assert_char(Register::RAX));

    let loop_label = format!("loop_{}", compiler.new_label_id());
    let end_label = format!("end_{}", compiler.new_label_id());
    let empty_label = format!("empty_{}", compiler.new_label_id());

    // Special case for empty string
    statements.push(Statement::Cmp {
        dest: R8,
        src: Operand::from(Value::Int(0)),
    });
    statements.push(Statement::Je {
        label: empty_label.clone(),
    });

    // Stash the top address and cast it to the string type.
    statements.push(Statement::Mov { dest: R9, src: RBX });
    statements.push(Statement::Or {
        dest: R9,
        src: Operand::Immediate(STRING_TYPE.tag.0 as i64),
    });

    // Put the length of the string at the beginning.
    // Note: The length is guaranteed to be an integer, so we can strip the type tag.
    statements.push(Statement::Sar {
        dest: R8,
        src: Operand::Immediate(INT_TYPE.shift as i64),
    });
    statements.push(Statement::Mov {
        dest: Operand::Offset(Register::RBX, 0),
        src: R8,
    });
    statements.push(Statement::Add {
        dest: RBX,
        src: Operand::Immediate(8),
    });

    // Each element of the string is guaranteed to be a character, so we can strip the type tag.
    statements.push(Statement::Sar {
        dest: RAX,
        src: Operand::Immediate(CHAR_TYPE.shift as i64),
    });

    // Unlike the vector, each element of the string only takes up 32 bits (4 bytes).
    // This breaks the 8-bytes alignment of the heap when the length is odd.
    // To fix this, we pad the array with extra 4 bytes if the length is odd.
    statements.push(Statement::Add {
        dest: R8,
        src: Operand::Immediate(1),
    });
    statements.push(Statement::Sar {
        dest: R8,
        src: Operand::Immediate(1),
    });
    statements.push(Statement::Sal {
        dest: R8,
        src: Operand::Immediate(1),
    });

    // Initialize each element of the string to the given value.
    statements.push(Statement::Label {
        name: loop_label.clone(),
    });
    statements.push(Statement::Mov {
        dest: Operand::Offset(Register::RBX, 0),
        src: EAX,
    });
    statements.push(Statement::Add {
        dest: RBX,
        src: Operand::Immediate(4), // 4 bytes per character
    });
    statements.push(Statement::Sub {
        dest: R8,
        src: Operand::Immediate(1),
    });
    statements.push(Statement::Cmp {
        dest: R8,
        src: Operand::Immediate(0),
    });
    statements.push(Statement::Jne {
        label: loop_label.clone(),
    });

    // Return the string.
    statements.push(Statement::Mov { dest: RAX, src: R9 });
    statements.push(Statement::Jmp {
        label: end_label.clone(),
    });

    // Special case for empty string
    statements.push(Statement::Label {
        name: empty_label.clone(),
    });
    statements.push(Statement::Mov {
        dest: RAX,
        src: Operand::from(Value::EmptyString),
    });

    statements.push(Statement::Label {
        name: end_label.clone(),
    });
    statements
}

fn compile_string_literal(string: &str) -> Vec<Statement> {
    if string.is_empty() {
        return compile_empty_string();
    }

    let mut statements = vec![];

    // Stash the top address and cast it to the string type.
    statements.push(Statement::Mov { dest: R9, src: RBX });
    statements.push(Statement::Or {
        dest: R9,
        src: Operand::Immediate(STRING_TYPE.tag.0 as i64),
    });

    // Put the length of the string at the beginning.
    // Note: The length is guaranteed to be an integer, so we can strip the type tag.
    statements.push(Statement::Mov {
        dest: RAX,
        src: Operand::Immediate(string.len() as i64),
    });
    statements.push(Statement::Mov {
        dest: Operand::Offset(Register::RBX, 0),
        src: RAX,
    });
    statements.push(Statement::Add {
        dest: RBX,
        src: Operand::Immediate(8),
    });

    for c in string.chars() {
        statements.push(Statement::Mov {
            dest: EAX,
            src: Operand::Immediate(c as i64), // Each element is guaranteed to be a character, so the type tag is not needed.
        });
        statements.push(Statement::Mov {
            dest: Operand::Offset(Register::RBX, 0),
            src: EAX,
        });
        statements.push(Statement::Add {
            dest: RBX,
            src: Operand::Immediate(4), // 4 bytes per character
        });
    }

    // Return the string.
    statements.push(Statement::Mov { dest: RAX, src: R9 });
    statements
}

fn compile_empty_string() -> Vec<Statement> {
    vec![Statement::Mov {
        dest: RAX,
        src: Operand::from(Value::EmptyString),
    }]
}

/// Returns instructions which sets rax to the element in the vector at the given index,
/// assuming the vector and the index is already given in r8 and rax respectively.
fn compile_vector_ref(_compiler: &mut Compiler) -> Vec<Statement> {
    let mut statements = assert_vector(Register::R8);
    statements.extend(assert_natural_number(Register::RAX));

    // Special case for empty vector
    statements.push(Statement::Cmp {
        dest: R8,
        src: Operand::from(Value::EmptyVector),
    });
    statements.push(Statement::Je {
        label: ERR_LABEL.to_string(),
    });

    // Cast r8 to the raw pointer address of the vector.
    statements.push(Statement::Xor {
        dest: R8,
        src: Operand::Immediate(VECTOR_TYPE.tag.0 as i64),
    });
    // Set r9 to the length of the vector.
    statements.push(Statement::Mov {
        dest: R9,
        src: Operand::Offset(Register::R8, 0),
    });
    // Cast rax to raw integer representing the index.
    statements.push(Statement::Sar {
        dest: RAX,
        src: Operand::Immediate(INT_TYPE.shift as i64),
    });

    // Check if the index is out of bounds. (length - 1 < index)
    statements.push(Statement::Sub {
        dest: R9,
        src: Operand::Immediate(1),
    });
    statements.push(Statement::Cmp { dest: R9, src: RAX });
    statements.push(Statement::Jl {
        label: ERR_LABEL.to_string(),
    });

    // Get the element at the given index.
    statements.push(Statement::Sal {
        dest: RAX,
        src: Operand::Immediate(3), // Each element takes up 2^3 bytes.
    });
    statements.push(Statement::Add { dest: R8, src: RAX });
    statements.push(Statement::Mov {
        dest: RAX,
        src: Operand::Offset(Register::R8, 8), // The offset 8 is required as the first word is the length.
    });

    statements
}

/// Returns instructions which sets rax to the character in the string at the given index,
/// assuming the string and the index is already given in r8 and rax respectively.
fn compile_string_ref(_compiler: &mut Compiler) -> Vec<Statement> {
    let mut statements = assert_string(Register::R8);
    statements.extend(assert_natural_number(Register::RAX));

    // Special case for empty string
    statements.push(Statement::Cmp {
        dest: R8,
        src: Operand::from(Value::EmptyString),
    });
    statements.push(Statement::Je {
        label: ERR_LABEL.to_string(),
    });

    // Cast r8 to the raw pointer address of the string.
    statements.push(Statement::Xor {
        dest: R8,
        src: Operand::Immediate(STRING_TYPE.tag.0 as i64),
    });
    // Set r9 to the length of the string.
    statements.push(Statement::Mov {
        dest: R9,
        src: Operand::Offset(Register::R8, 0),
    });
    // Cast rax to raw integer representing the index.
    statements.push(Statement::Sar {
        dest: RAX,
        src: Operand::Immediate(INT_TYPE.shift as i64),
    });

    // Check if the index is out of bounds. (length - 1 < index)
    statements.push(Statement::Sub {
        dest: R9,
        src: Operand::Immediate(1),
    });
    statements.push(Statement::Cmp { dest: R9, src: RAX });
    statements.push(Statement::Jl {
        label: ERR_LABEL.to_string(),
    });

    // Get the element at the given index.
    statements.push(Statement::Sal {
        dest: RAX,
        src: Operand::Immediate(2), // Each element takes up 2^2 bytes.
    });
    statements.push(Statement::Add { dest: R8, src: RAX });
    statements.push(Statement::Mov {
        dest: EAX,
        src: Operand::Offset(Register::R8, 8), // The offset 8 is required as the first word is the length.
    });

    // Cast rax to the character type.
    statements.push(Statement::Sal {
        dest: RAX,
        src: Operand::Immediate(CHAR_TYPE.shift as i64),
    });
    statements.push(Statement::Xor {
        dest: RAX,
        src: Operand::Immediate(CHAR_TYPE.tag.0 as i64),
    });

    statements
}

/// Returns instructions which sets rax to true if the comparison flag is equal.
fn if_equal() -> Vec<Statement> {
    vec![
        Statement::Mov {
            dest: RAX,
            src: Operand::Immediate(Value::Boolean(false).encode()),
        },
        Statement::Mov {
            dest: R9,
            src: Operand::Immediate(Value::Boolean(true).encode()),
        },
        Statement::Cmove { dest: RAX, src: R9 },
    ]
}

/// Returns instructions which sets rax to true if the comparison flag is less
fn if_less_than() -> Vec<Statement> {
    vec![
        Statement::Mov {
            dest: RAX,
            src: Operand::Immediate(Value::Boolean(false).encode()),
        },
        Statement::Mov {
            dest: R9,
            src: Operand::Immediate(Value::Boolean(true).encode()),
        },
        Statement::Cmovl { dest: RAX, src: R9 },
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
        dest: RAX,
        src: Operand::from(Value::Boolean(false)),
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

fn compile_let(expr: ast::Let, compiler: &mut Compiler) -> Vec<Statement> {
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

fn compile_variable(variable: Variable, compiler: &mut Compiler) -> Vec<Statement> {
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

fn cast_type(register: Register, from: &UnaryType, to: &UnaryType) -> Vec<Statement> {
    vec![
        Statement::Sar {
            dest: Operand::Register(register.clone()),
            src: Operand::Immediate(from.shift as i64),
        },
        Statement::Sal {
            dest: Operand::Register(register.clone()),
            src: Operand::Immediate(to.shift as i64),
        },
        Statement::Xor {
            dest: Operand::Register(register),
            src: Operand::Immediate(to.tag.0 as i64),
        },
    ]
}

/// Returns instructions which sets rax to true iff the value in rax is of the given type.
fn is_type(type_: &UnaryType) -> Vec<Statement> {
    let mut statements = vec![
        Statement::And {
            dest: RAX,
            src: Operand::Immediate(type_.mask() as i64),
        },
        Statement::Cmp {
            dest: RAX,
            src: Operand::Immediate(type_.tag.0 as i64),
        },
    ];
    statements.extend(if_equal());
    statements
}

fn assert_type(register: Register, type_: &UnaryType) -> Vec<Statement> {
    vec![
        Statement::Mov {
            dest: R9,
            src: Operand::Register(register),
        },
        Statement::And {
            dest: R9,
            src: Operand::Immediate(type_.mask() as i64),
        },
        Statement::Cmp {
            dest: R9,
            src: Operand::Immediate(type_.tag.0 as i64),
        },
        Statement::Jne {
            label: ERR_LABEL.to_string(),
        },
    ]
}

fn assert_int(register: Register) -> Vec<Statement> {
    assert_type(register, &INT_TYPE)
}

fn assert_char(register: Register) -> Vec<Statement> {
    assert_type(register, &CHAR_TYPE)
}

fn assert_box(register: Register) -> Vec<Statement> {
    assert_type(register, &BOX_TYPE)
}

fn assert_cons(register: Register) -> Vec<Statement> {
    assert_type(register, &CONS_TYPE)
}

fn assert_vector(register: Register) -> Vec<Statement> {
    assert_type(register, &VECTOR_TYPE)
}

fn assert_string(register: Register) -> Vec<Statement> {
    assert_type(register, &STRING_TYPE)
}

fn assert_natural_number(register: Register) -> Vec<Statement> {
    let mut statements = assert_int(register.clone());
    statements.push(Statement::Cmp {
        dest: Operand::Register(register),
        src: Operand::from(Value::Int(0)),
    });
    statements.push(Statement::Jl {
        label: ERR_LABEL.to_string(),
    });
    statements
}

fn assert_codepoint() -> Vec<Statement> {
    let mut statements = assert_int(Register::RAX);

    // Make sure the value is in the range 0..=0x10FFFF
    statements.push(Statement::Cmp {
        dest: RAX,
        src: Operand::from(Value::Int(0)),
    });
    statements.push(Statement::Jl {
        label: ERR_LABEL.to_string(),
    });
    statements.push(Statement::Cmp {
        dest: RAX,
        src: Operand::from(Value::Int(0x10FFFF)),
    });
    statements.push(Statement::Jg {
        label: ERR_LABEL.to_string(),
    });

    // except for the range 55296..=57343.
    statements.push(Statement::Cmp {
        dest: RAX,
        src: Operand::from(Value::Int(55295)),
    });
    statements.push(Statement::Jl {
        label: "ok".to_string(),
    });
    statements.push(Statement::Cmp {
        dest: RAX,
        src: Operand::from(Value::Int(57344)),
    });
    statements.push(Statement::Jg {
        label: "ok".to_string(),
    });
    statements.push(Statement::Jmp {
        label: ERR_LABEL.to_string(),
    });
    statements.push(Statement::Label {
        name: "ok".to_string(),
    });
    statements
}

fn assert_byte(register: Register) -> Vec<Statement> {
    let mut statements = assert_int(register);

    // Make sure the value is in the range 0..=255
    statements.push(Statement::Cmp {
        dest: R9,
        src: Operand::from(Value::Int(0)),
    });
    statements.push(Statement::Jl {
        label: ERR_LABEL.to_string(),
    });
    statements.push(Statement::Cmp {
        dest: R9,
        src: Operand::from(Value::Int(255)),
    });
    statements.push(Statement::Jg {
        label: ERR_LABEL.to_string(),
    });

    statements
}

fn call(label: String) -> Vec<Statement> {
    let mut statements = pad_stack();
    statements.push(Statement::Call { label });
    statements.extend(unpad_stack());
    statements
}

/// Returns instructions which alligns the stack pointer to a 16-byte boundary.
/// This must be done before calling an external function.
/// After the call, the stack must be unaligned using [unpad_stack].
fn pad_stack() -> Vec<Statement> {
    vec![
        Statement::Mov {
            dest: R15,
            src: RSP,
        },
        Statement::And {
            dest: R15,
            src: Operand::Immediate(0b1000),
        },
        Statement::Sub {
            dest: RSP,
            src: R15,
        },
    ]
}

/// Returns instructions which undo the stack alignment done by [pad_stack].
fn unpad_stack() -> Vec<Statement> {
    vec![Statement::Add {
        dest: RSP,
        src: R15,
    }]
}

impl From<Value> for Operand {
    fn from(value: Value) -> Self {
        Operand::Immediate(value.encode())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn variable_position() {
        let mut variables_table = VariablesTable::new();
        variables_table.push_variable(Variable("a".to_string()));
        variables_table.push_variable(Variable("b".to_string()));
        variables_table.push_variable(Variable("c".to_string()));
        variables_table.pop();
        assert_eq!(
            variables_table.position(&Variable("a".to_string())),
            Some(1)
        );
        assert_eq!(
            variables_table.position(&Variable("b".to_string())),
            Some(0)
        );
        assert_eq!(variables_table.position(&Variable("c".to_string())), None);
    }
}
