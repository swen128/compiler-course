use std::collections::HashSet;

use crate::{
    a86::ast::{Operand, Register, Statement},
    mylang::{
        ast::{App, Expr, If, Lambda, Let, Lit, Match, Program},
        data_type::*,
    },
};

use super::{
    error::ERR_LABEL,
    state::{Compiler, Label},
    types::*,
};

const RAX: Operand = Operand::Register(Register::RAX);
const RBX: Operand = Operand::Register(Register::RBX);
const EAX: Operand = Operand::Register(Register::EAX);
const R8: Operand = Operand::Register(Register::R8);
const R9: Operand = Operand::Register(Register::R9);
const R9D: Operand = Operand::Register(Register::R9D);

/// Returns instructions to initialize a string of the given length with the repeated values,
/// assuming the length and the value is already given in r8 and rax respectively.
pub fn compile_make_string(compiler: &mut Compiler) -> Vec<Statement> {
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

/// Returns instructions which sets rax to the character in the string at the given index,
/// assuming the string and the index is already given in r8 and rax respectively.
pub fn compile_string_ref(_compiler: &mut Compiler) -> Vec<Statement> {
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

/// Returns instructions which sets the comparison flag to true iff
/// value in rax equals to the given static string.
pub fn compare_strings(string: &str, compiler: &mut Compiler) -> Vec<Statement> {
    let neq_label = format!("string_neq_{}", compiler.new_label_id());
    let jne = Statement::Jne {
        label: neq_label.clone(),
    };

    let mut statements = vec![
        // Check if the type of rax is string.
        Statement::Mov { dest: R9, src: RAX },
        Statement::And {
            dest: R9,
            src: Operand::Immediate(STRING_TYPE.mask() as i64),
        },
        Statement::Cmp {
            dest: R9,
            src: Operand::Immediate(STRING_TYPE.tag.0 as i64),
        },
        jne.clone(),
        // Set r8 to the raw pointer address of the string.
        Statement::Mov { dest: R8, src: RAX },
        Statement::Xor {
            dest: R8,
            src: Operand::Immediate(STRING_TYPE.tag.0 as i64),
        },
        // Set r9 to the length of the string.
        Statement::Mov {
            dest: R9,
            src: Operand::Offset(Register::R8, 0),
        },
        // Compare the length.
        Statement::Cmp {
            dest: R9,
            src: Operand::Immediate(string.len() as i64),
        },
    ];

    // Compare each character.
    for (i, c) in string.chars().enumerate() {
        statements.push(Statement::Mov {
            dest: R9D,
            src: Operand::Immediate(c as i64), // Each element is guaranteed to be a character, so the type tag is not needed.
        });
        statements.push(Statement::Cmp {
            dest: Operand::Offset(Register::R8, 8 + 4 * i as i64), // 8 bytes for the length, 4 bytes per character
            src: R9D,
        });
        statements.push(jne.clone());
    }

    statements.push(Statement::Label { name: neq_label });

    statements
}

/// Returns instructions which sets rax to the given string literal.
pub fn compile_string_literal(string: &str, compiler: &Compiler) -> Vec<Statement> {
    let Label(label) = compiler
        .string_literal_label(string)
        .expect(format!("String literal '{}' not found in the table", string).as_str());

    // Tag the address of the label as the string type.
    vec![Statement::LeaArithmetic {
        dest: RAX,
        expr: format!("[{} + {}]", label, STRING_TYPE.tag.0),
    }]
}

/// Returns pseudo-instructions declaring static data for all string literals in the program.
pub fn compile_all_string_data(compiler: &Compiler) -> Vec<Statement> {
    let mut statements = vec![];
    for (string, label) in compiler.string_literals() {
        statements.push(Statement::Label {
            name: label.0.clone(),
        });
        statements.extend(compile_string_data(string));
    }
    statements
}

/// Returns pseudo-instructions declaring compile-time static string data,
/// which should be put in the data section.
fn compile_string_data(string: &str) -> Vec<Statement> {
    let mut statements = vec![Statement::Dq {
        value: string.len() as i64,
    }];
    statements.extend(string.chars().map(|c| Statement::Dd { value: c as i32 }));
    statements
}

pub fn all_string_literals(program: &Program) -> HashSet<String> {
    let mut result = HashSet::new();
    for function_definition in &program.function_definitions {
        result.extend(string_literals(&function_definition.body));
    }
    result.extend(string_literals(&program.expr));
    result
}

fn string_literals(expr: &Expr) -> HashSet<String> {
    match expr {
        Expr::Lit(Lit::String(s)) => {
            let mut result = HashSet::new();
            result.insert(s.clone());
            result
        }

        // Just recurse into subexpressions.
        Expr::Prim1(_, e) => string_literals(e),
        Expr::Prim2(_, e1, e2) => {
            let mut result = string_literals(e1);
            result.extend(string_literals(e2));
            result
        }
        Expr::Prim3(_, e1, e2, e3) => {
            let mut result = string_literals(e1);
            result.extend(string_literals(e2));
            result.extend(string_literals(e3));
            result
        }
        Expr::Begin(e1, e2) => {
            let mut result = string_literals(e1);
            result.extend(string_literals(e2));
            result
        }
        Expr::App(App { function, args }) => {
            let mut result = string_literals(function);
            for arg in args {
                result.extend(string_literals(arg));
            }
            result
        }
        Expr::If(If { cond, then, els }) => {
            let mut result = string_literals(cond);
            result.extend(string_literals(then));
            result.extend(string_literals(els));
            result
        }
        Expr::Match(Match { expr, arms }) => {
            let mut result = string_literals(expr);
            for arm in arms {
                result.extend(string_literals(&arm.body));
            }
            result
        }
        Expr::Let(Let { binding, body }) => {
            let mut result = string_literals(&binding.rhs);
            result.extend(string_literals(body));
            result
        }
        Expr::Lambda(Lambda {
            id: _,
            params: _,
            body,
        }) => string_literals(body),

        Expr::Lit(_) | Expr::Variable(_) | Expr::Eof | Expr::Prim0(_) => HashSet::new(),
    }
}
