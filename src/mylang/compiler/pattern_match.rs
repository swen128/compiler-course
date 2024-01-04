use crate::{
    a86::ast::{Operand, Register, Statement},
    mylang::{
        ast::{self, Identifier},
        data_type::{Value, BOX_TYPE, CONS_TYPE},
    },
};

use super::{
    error::ERR_LABEL, expr::compile_expr, state::Compiler, string::compare_strings,
    variable::VariablesTable,
};

const RAX: Operand = Operand::Register(Register::RAX);
const RSP: Operand = Operand::Register(Register::RSP);
const R8: Operand = Operand::Register(Register::R8);
const R9: Operand = Operand::Register(Register::R9);

pub fn compile_match(
    match_expr: ast::Match,
    compiler: &mut Compiler,
    env: &VariablesTable,
    is_tail_expr: bool,
) -> Vec<Statement> {
    let mut statements = compile_expr(*match_expr.expr, compiler, env, false);
    let done_label = format!("done_{}", compiler.new_label_id());

    for arm in match_expr.arms {
        statements.extend(compile_match_arm(
            arm,
            compiler,
            env,
            is_tail_expr,
            &done_label,
        ));
    }

    // In case of no match, raise an error.
    statements.push(Statement::Jmp {
        label: ERR_LABEL.to_string(),
    });

    statements.push(Statement::Label { name: done_label });

    statements
}

fn compile_match_arm(
    arm: ast::Arm,
    compiler: &mut Compiler,
    env: &VariablesTable,
    is_tail_expr: bool,
    done_label: &str,
) -> Vec<Statement> {
    let next_label = format!("next_{}", compiler.new_label_id());
    let CompiledPattern {
        mut statements,
        env: bound_env,
    } = compile_pattern(arm.pattern, compiler, env, &next_label);

    statements.extend(compile_expr(*arm.body, compiler, &bound_env, is_tail_expr));

    // Clear the variables bound in the match scope before finishing.
    statements.push(Statement::Add {
        dest: RSP,
        src: Operand::Immediate(8 * (bound_env.len() - env.len()) as i64),
    });
    statements.push(Statement::Jmp {
        label: done_label.to_string(),
    });

    statements.push(Statement::Label {
        name: next_label.to_string(),
    });

    statements
}

struct CompiledPattern {
    /// Instructions to determine if the value in rax matches the pattern,
    /// and pushes bound variables to the stack if applicable.
    statements: Vec<Statement>,

    /// Variable bindings in case the pattern matches.
    env: VariablesTable,
}

impl CompiledPattern {
    fn new(statements: Vec<Statement>, env: VariablesTable) -> Self {
        Self { statements, env }
    }
}

fn compile_pattern(
    pattern: ast::Pattern,
    compiler: &mut Compiler,
    env: &VariablesTable,
    next_label: &str,
) -> CompiledPattern {
    match pattern {
        ast::Pattern::Wildcard => compile_wildcard_pattern(env),
        ast::Pattern::Variable(identifier) => compile_variable_pattern(&identifier, env),
        ast::Pattern::Lit(lit) => compile_literal_pattern(lit, compiler, env, next_label),

        ast::Pattern::Cons(car, cdr) => compile_cons_pattern(*car, *cdr, compiler, env, next_label),
        ast::Pattern::Box(pattern) => compile_box_pattern(*pattern, compiler, env, next_label),
        ast::Pattern::And(left, right) => {
            compile_and_pattern(*left, *right, compiler, env, next_label)
        }
    }
}

fn compile_wildcard_pattern(env: &VariablesTable) -> CompiledPattern {
    // It always matches and bind nothing.
    CompiledPattern::new(vec![], env.clone())
}

fn compile_variable_pattern(identifier: &Identifier, env: &VariablesTable) -> CompiledPattern {
    // It always matches and bind the variable.
    CompiledPattern::new(vec![Statement::Push { src: RAX }], env.with_var(identifier))
}

fn compile_literal_pattern(
    lit: ast::Lit,
    compiler: &mut Compiler,
    env: &VariablesTable,
    next_label: &str,
) -> CompiledPattern {
    let matched_label = format!("matched_{}", compiler.new_label_id());

    let mut statements = compare_literal(lit, compiler);
    statements.push(Statement::Je {
        label: matched_label.clone(),
    });

    // No match, clear the stack and jump to the next pattern.
    statements.push(Statement::Add {
        dest: RSP,
        src: Operand::Immediate(8 * env.len() as i64),
    });
    statements.push(Statement::Jmp {
        label: next_label.to_string(),
    });

    // Matched
    statements.push(Statement::Label {
        name: matched_label.clone(),
    });

    CompiledPattern::new(statements, env.clone())
}

/// Returns instructions which sets the comparison flag to true iff
/// value in rax equals to the given literal.
fn compare_literal(lit: ast::Lit, compiler: &mut Compiler) -> Vec<Statement> {
    fn cmp(operand: Operand) -> Vec<Statement> {
        vec![Statement::Cmp {
            dest: RAX,
            src: operand,
        }]
    }
    match lit {
        ast::Lit::Int(i) => cmp(Operand::from(Value::Int(i))),
        ast::Lit::Bool(b) => cmp(Operand::from(Value::Boolean(b))),
        ast::Lit::Char(c) => cmp(Operand::from(Value::Char(c))),
        ast::Lit::EmptyList => cmp(Operand::from(Value::EmptyList)),
        ast::Lit::String(s) => compare_strings(&s, compiler),
    }
}

fn compile_box_pattern(
    pattern: ast::Pattern,
    compiler: &mut Compiler,
    env: &VariablesTable,
    next_label: &str,
) -> CompiledPattern {
    let matched_label = format!("matched_{}", compiler.new_label_id());

    let mut statements = vec![
        // Check if the type of rax is box.
        Statement::Mov { dest: R9, src: RAX },
        Statement::And {
            dest: R9,
            src: Operand::Immediate(BOX_TYPE.mask() as i64),
        },
        Statement::Cmp {
            dest: R9,
            src: Operand::Immediate(BOX_TYPE.tag.0 as i64),
        },
        Statement::Je {
            label: matched_label.clone(),
        },
    ];

    // No match, clear the stack and jump to the next pattern.
    statements.push(Statement::Add {
        dest: RSP,
        src: Operand::Immediate(8 * env.len() as i64),
    });
    statements.push(Statement::Jmp {
        label: next_label.to_string(),
    });

    // Matched, unbox it for subpattern matching.
    statements.push(Statement::Label {
        name: matched_label.clone(),
    });
    statements.push(Statement::Xor {
        dest: RAX,
        src: Operand::Immediate(BOX_TYPE.tag.0 as i64),
    });
    statements.push(Statement::Mov {
        dest: RAX,
        src: Operand::Offset(Register::RAX, 0),
    });

    let subpattern = compile_pattern(pattern, compiler, env, next_label);
    statements.extend(subpattern.statements);

    CompiledPattern::new(statements, subpattern.env)
}

fn compile_cons_pattern(
    car: ast::Pattern,
    cdr: ast::Pattern,
    compiler: &mut Compiler,
    env: &VariablesTable,
    next_label: &str,
) -> CompiledPattern {
    let matched_label = format!("matched_{}", compiler.new_label_id());

    let mut statements = vec![
        // Check if the type of rax is cons.
        Statement::Mov { dest: R9, src: RAX },
        Statement::And {
            dest: R9,
            src: Operand::Immediate(CONS_TYPE.mask() as i64),
        },
        Statement::Cmp {
            dest: R9,
            src: Operand::Immediate(CONS_TYPE.tag.0 as i64),
        },
        Statement::Je {
            label: matched_label.clone(),
        },
    ];

    // No match, clear the stack and jump to the next pattern.
    statements.push(Statement::Add {
        dest: RSP,
        src: Operand::Immediate(8 * env.len() as i64),
    });
    statements.push(Statement::Jmp {
        label: next_label.to_string(),
    });

    // Matched, cast it to the raw pointer address.
    statements.push(Statement::Label {
        name: matched_label.clone(),
    });
    statements.push(Statement::Xor {
        dest: RAX,
        src: Operand::Immediate(CONS_TYPE.tag.0 as i64),
    });

    // Stash cdr to the stack.
    statements.push(Statement::Mov {
        dest: R8,
        src: Operand::Offset(Register::RAX, 0),
    });
    statements.push(Statement::Push { src: R8 });
    let env = &env.with_non_var();

    // Check if car matches the pattern.
    statements.push(Statement::Mov {
        dest: RAX,
        src: Operand::Offset(Register::RAX, 8),
    });
    let car_result = compile_pattern(car, compiler, env, next_label);
    statements.extend(car_result.statements);

    // Check if cdr matches the pattern.
    statements.push(Statement::Mov {
        dest: RAX,
        // Make sure to account for the newly pushed variables by the car pattern matching.
        src: Operand::Offset(Register::RSP, 8 * (car_result.env.len() - env.len()) as i64),
    });
    let cdr_result = compile_pattern(cdr, compiler, &car_result.env, next_label);
    statements.extend(cdr_result.statements);

    CompiledPattern::new(statements, cdr_result.env)
}

fn compile_and_pattern(
    left: ast::Pattern,
    right: ast::Pattern,
    compiler: &mut Compiler,
    env: &VariablesTable,
    next_label: &str,
) -> CompiledPattern {
    // Stash the value in rax to the stack.
    let mut statements = vec![
        Statement::Push { src: RAX },
    ];
    let env = &env.with_non_var();

    // Check if the left pattern matches.
    let left_result = compile_pattern(left, compiler, env, next_label);
    statements.extend(left_result.statements);
    
    // Check if the right pattern matches.
    statements.push(Statement::Mov {
        dest: RAX,
        // Make sure to account for the newly pushed variables by the left pattern matching.
        src: Operand::Offset(Register::RSP, 8 * (left_result.env.len() - env.len()) as i64),
    });
    let right_result = compile_pattern(right, compiler, &left_result.env, next_label);
    statements.extend(right_result.statements);

    CompiledPattern::new(statements, right_result.env)
}
