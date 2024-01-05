//! # Function calling convention
//!
//! The caller pushes the return address, closure pointer and arguments to the stack before calling function.
//! The callee clears the stack before returning.
//!
//! So for example, when you call `lambda (x y z) ...`, the stack should look like:
//!
//! ```plaintext
//! + ---------------------+
//! |   return address     |
//! +----------------------+
//! |   closure pointer    |
//! +----------------------+
//! |       x              |
//! +----------------------+
//! |       y              |
//! +----------------------+
//! |       z              | <-- rsp
//! +----------------------+
//! ```
//!
//! Note the last argument `z` is at the top of the stack.
//!
//! The closure should point to a region of the heap memory containing captured variables.
//! For an expression `let ((a 1) (b 2)) (lambda (x y z) (+ a b x y z))`, the heap should look like this:
//!
//! ```plaintext
//! +----------------------+
//! |       b              |
//! +----------------------+`
//! |       a              |
//! +----------------------+
//! | address of the label | <-- closure pointer
//! +----------------------+
//! ```

use std::collections::HashSet;

use super::{expr::compile_expr, state::Compiler, types::assert_closure, variable::VariablesTable};
use crate::{
    a86::ast::{Operand, Register, Statement},
    mylang::{
        ast::{App, Expr, FunctionDefinition, Identifier, If, Lambda, Let, Match, Program},
        data_type::CLOSURE_TYPE,
    },
};

const RAX: Operand = Operand::Register(Register::RAX);
const RBX: Operand = Operand::Register(Register::RBX);
const R8: Operand = Operand::Register(Register::R8);

pub fn compile_defines(program: &Program, compiler: &mut Compiler) -> Vec<Statement> {
    program
        .function_definitions
        .iter()
        .flat_map(|f| compile_define(f.clone(), compiler))
        .collect()
}

pub fn defined_ids(program: &Program) -> Vec<Identifier> {
    program
        .function_definitions
        .iter()
        .map(|f| f.signature.name.clone())
        .collect()
}

fn compile_define(
    function_definition: FunctionDefinition,
    compiler: &mut Compiler,
) -> Vec<Statement> {
    compile_lambda_definition(function_definition.into(), compiler)
}

/// Returns instructions initializing closures for all functions declared by the `define` keyword.
pub fn compile_closures_for_defines(program: &Program) -> Vec<Statement> {
    let definitions = &program.function_definitions;

    // We have to create closure values in the following two steps,
    // because they may have cyclic references to each other.
    // We need to know the addresses of all closures before we can initialize them.
    let mut statements = allocate_closures(definitions);
    let env = VariablesTable::new().extended(defined_ids(program));
    statements.extend(init_closures(definitions, &env));

    // Adavance the heap pointer.
    let size: usize = definitions
        .iter()
        .map(|f| allocated_heap_size(&f.clone().into()))
        .sum();
    statements.push(Statement::Add {
        dest: RBX,
        src: Operand::Immediate(size as i64),
    });

    statements
}

/// Returns instructions allocating heap memory for closures, but not capturing free variables yet.
/// It also pushes addresses of closures to the stack.
fn allocate_closures(definitions: &Vec<FunctionDefinition>) -> Vec<Statement> {
    let mut statements = vec![];

    let mut offset = 0;
    for definition in definitions {
        let lambda: Lambda = definition.clone().into();

        // Write the address of the label to the heap.
        statements.push(Statement::Lea {
            dest: RAX,
            label: get_label(&lambda),
        });
        statements.push(Statement::Mov {
            dest: Operand::Offset(Register::RBX, offset),
            src: RAX,
        });

        // Push the address of the closure to the stack.
        statements.push(Statement::Mov {
            dest: RAX,
            src: RBX,
        });
        statements.push(Statement::Add {
            dest: RAX,
            src: Operand::Immediate(offset as i64),
        });
        statements.push(Statement::Or {
            dest: RAX,
            src: Operand::Immediate(CLOSURE_TYPE.tag.0 as i64),
        });
        statements.push(Statement::Push { src: RAX });

        offset += allocated_heap_size(&lambda) as i64;
    }

    statements
}

fn init_closures(definitions: &Vec<FunctionDefinition>, env: &VariablesTable) -> Vec<Statement> {
    let mut statements = vec![];

    let mut offset = 8;
    for definition in definitions {
        let lambda: Lambda = definition.clone().into();
        let free_vars = free_variables(&Expr::Lambda(lambda.clone()));

        statements.extend(capture_variables(&free_vars, env, offset));

        offset += allocated_heap_size(&lambda);
    }

    statements
}

/// Returns instructions defining all lambda expressions in the program.
///
/// It should be put after the main program and invoked by [compile_function_application].
pub fn compile_lambda_definitions(program: &Program, compiler: &mut Compiler) -> Vec<Statement> {
    all_lambdas(program)
        .into_iter()
        .flat_map(|lambda| compile_lambda_definition(lambda, compiler))
        .collect()
}

/// Returns labeled instructions defining the lambda expression.
///
/// It assumes the caller has pushed the return address, closure pointer and arguments to the stack.
pub fn compile_lambda_definition(lambda: Lambda, compiler: &mut Compiler) -> Vec<Statement> {
    let params = lambda.params.clone();
    let body = lambda.body.clone();

    let mut statements = vec![
        Statement::Label {
            name: get_label(&lambda),
        },
        // Set rax to the closure pointer.
        Statement::Mov {
            dest: RAX,
            src: Operand::Offset(Register::RSP, 8 * params.len() as i64),
        },
        Statement::Xor {
            dest: RAX,
            src: Operand::Immediate(CLOSURE_TYPE.tag.0 as i64),
        },
    ];

    // Copy the captured variables from the heap to the stack.
    // After this, the stack should look like:
    //
    // +----------------------+
    // |   return address     |
    // +----------------------+
    // |   closure pointer    |
    // +----------------------+
    // |       x              |
    // +----------------------+
    // |       y              |
    // +----------------------+
    // |       z              |
    // +----------------------+
    // |       a              |
    // +----------------------+
    // |       b              | <-- rsp
    // +----------------------+
    let free_vars = free_variables(&Expr::Lambda(lambda.clone()));
    for i in 0..free_vars.len() {
        // The value at [rax] holds the address of the label.
        // [rax + 8 + 8 * i] holds the i-th free variable.
        let offset = 8 + 8 * i as i64;
        statements.push(Statement::Mov {
            dest: R8,
            src: Operand::Offset(Register::RAX, offset),
        });
        statements.push(Statement::Push { src: R8 });
    }

    // Now that the environment is set up, compile the body of the lambda expression.
    let env = VariablesTable::new()
        .with_non_var() // Corresponding to the closure pointer.
        .extended(params)
        .extended(free_vars);
    statements.extend(compile_expr(*body, compiler, &env, true));

    // Clear the stack.
    statements.push(Statement::Add {
        dest: Operand::Register(Register::RSP),
        src: Operand::Immediate(8 * env.len() as i64),
    });

    statements.push(Statement::Ret);

    statements
}

/// Returns instructions which create a closure data and set rax to its tagged pointer.
pub fn compile_closure(lambda: Lambda, env: &VariablesTable) -> Vec<Statement> {
    // A closure is a fixed-size sequence of values in the heap.

    // The first value is the address to the label of the lambda expression.
    let mut statements = vec![
        Statement::Lea {
            dest: RAX,
            label: get_label(&lambda),
        },
        Statement::Mov {
            dest: Operand::Offset(Register::RBX, 0),
            src: RAX,
        },
    ];

    // The rest of the sequence captures free variables in the current environment.
    let free_vars = free_variables(&Expr::Lambda(lambda.clone()));
    statements.extend(capture_variables(&free_vars, env, 8));

    // Set rax to the tagged pointer.
    statements.push(Statement::Mov {
        dest: RAX,
        src: RBX,
    });
    statements.push(Statement::Or {
        dest: RAX,
        src: Operand::Immediate(CLOSURE_TYPE.tag.0 as i64),
    });

    // Advance the heap pointer.
    statements.push(Statement::Add {
        dest: RBX,
        src: Operand::Immediate(8 * (free_vars.len() + 1) as i64),
    });

    statements
}

/// Returns instructions which copy the given variables to the memory area starting from [rbx + offset].
fn capture_variables(
    variables: &HashSet<Identifier>,
    env: &VariablesTable,
    offset: usize,
) -> Vec<Statement> {
    let mut statements = vec![];

    for (i, variable) in variables.iter().enumerate() {
        let current_offset = (offset as i64) + 8 * i as i64;
        let lexical_address = env
            .position(variable)
            .map(|pos| 8 * pos as i64)
            .expect(format!("Undefined variable `{}`", variable.0).as_str()); // TODO: Return `Result` type.

        statements.push(Statement::Mov {
            dest: R8,
            src: Operand::Offset(Register::RSP, lexical_address),
        });
        statements.push(Statement::Mov {
            dest: Operand::Offset(Register::RBX, current_offset),
            src: R8,
        });
    }

    statements
}

/// Returns the size of the heap allocated for closure.
fn allocated_heap_size(lambda: &Lambda) -> usize {
    // 1 word for the label address, and 1 word for each free variable.
    8 + 8 * free_variables(&Expr::Lambda(lambda.clone())).len()
}

/// Returns instrcutions to invoke a function defined by
/// [compile_function_definition] or [compile_lambda_definition].
pub fn compile_function_application(
    function_application: App,
    compiler: &mut Compiler,
    env: &VariablesTable,
    is_tail_expr: bool,
) -> Vec<Statement> {
    if is_tail_expr {
        compile_function_application_tail(function_application, compiler, env)
    } else {
        compile_function_application_non_tail(function_application, compiler, env)
    }
}

fn compile_function_application_non_tail(
    function_application: App,
    compiler: &mut Compiler,
    env: &VariablesTable,
) -> Vec<Statement> {
    let return_label = format!("function_return_site_{}", compiler.new_label_id());

    let App { function, args } = function_application;
    let n_args = args.len() as i64;

    // If you used the `call` instruction, the return address is put at the top of the stack,
    // thus violating the calling convention.
    // In order to avoid this, we use more primitive instructions to push the return address first.
    let mut statements = vec![
        Statement::Lea {
            dest: RAX,
            label: return_label.clone(),
        },
        Statement::Push { src: RAX },
    ];
    let mut env = env.with_non_var();

    // Push the function to the stack.
    statements.extend(compile_expr(*function, compiler, &env, false));
    statements.extend(assert_closure(Register::RAX));
    statements.push(Statement::Push { src: RAX });

    // And then push the arguments to the stack.
    for arg in args {
        env = env.with_non_var();
        statements.extend(compile_expr(arg, compiler, &env, false));
        statements.push(Statement::Push { src: RAX });
    }

    // Set rax to the address of the function label.
    statements.push(Statement::Mov {
        dest: RAX,
        src: Operand::Offset(Register::RSP, 8 * n_args),
    });
    statements.push(Statement::Xor {
        dest: RAX,
        src: Operand::Immediate(CLOSURE_TYPE.tag.0 as i64),
    });
    statements.push(Statement::Mov {
        dest: RAX,
        src: Operand::Offset(Register::RAX, 0),
    });

    statements.push(Statement::JmpRegister(Register::RAX));
    statements.push(Statement::Label { name: return_label });

    statements
}

/// Returns optimized instructions for tail calls.
fn compile_function_application_tail(
    function_application: App,
    compiler: &mut Compiler,
    env: &VariablesTable,
) -> Vec<Statement> {
    let App { function, args } = function_application;
    let n_args = args.len();
    let n_env = env.len();

    let mut statements = vec![];

    // Push the function to the stack.
    statements.extend(compile_expr(*function, compiler, &env, false));
    statements.extend(assert_closure(Register::RAX));
    statements.push(Statement::Push { src: RAX });

    // And then push the arguments to the stack.
    let mut new_env = env.clone();
    for arg in args {
        new_env = new_env.with_non_var();
        statements.extend(compile_expr(arg, compiler, &new_env, false));
        statements.push(Statement::Push { src: RAX });
    }

    // Let’s say we have an expression that looks like this:
    //
    // ```racket
    // (let ((x 1))
    //   (let ((y 2))
    //     (f (+ x y) 5)))
    // ```
    //
    // Then `env` here has variables `x` and `y`.
    // After `args` (which are 1+2 and 5) are pushed to the stack, it will look like:
    //         + ---------------------+
    //         |   return address     |
    //         +----------------------+
    //         |   x : 1              |
    //         +----------------------+
    //         |   y : 2              |
    //         +----------------------+
    //         |   f : closure        |
    //         +----------------------+
    //         |       3              |
    //         +----------------------+
    // rsp --> |       5              |
    //         +----------------------+
    //
    // As this is tail call, we no longer need `x` and `y`.
    // We want the stack like:
    //         + ---------------------+
    //         |   return address     |
    //         +----------------------+
    //         |   f : closure        |
    //         +----------------------+
    //         |       3              |
    //         +----------------------+
    // rsp --> |       5              |
    //         +----------------------+
    //
    // The `move_args` function does just that.
    statements.extend(move_args(n_args + 1, n_env));

    // Set rax to the address of the function label.
    statements.push(Statement::Mov {
        dest: RAX,
        src: Operand::Offset(Register::RSP, 8 * n_args as i64),
    });
    statements.push(Statement::Xor {
        dest: RAX,
        src: Operand::Immediate(CLOSURE_TYPE.tag.0 as i64),
    });
    statements.push(Statement::Mov {
        dest: RAX,
        src: Operand::Offset(Register::RAX, 0),
    });

    statements.push(Statement::JmpRegister(Register::RAX));

    statements
}

/// Moves [n_args] top elements on the stack up by [offset] words.
/// This should be used in [compile_function_application_tail].
fn move_args(n_args: usize, offset: usize) -> Vec<Statement> {
    let mut statements = vec![];

    if n_args == 0 || offset == 0 {
        return statements;
    }

    for i in (0..n_args).rev() {
        statements.push(Statement::Mov {
            dest: R8,
            src: Operand::Offset(Register::RSP, i as i64 * 8),
        });
        statements.push(Statement::Mov {
            dest: Operand::Offset(Register::RSP, (i + offset) as i64 * 8),
            src: R8,
        });
    }
    statements.push(Statement::Add {
        dest: Operand::Register(Register::RSP),
        src: Operand::Immediate(offset as i64 * 8),
    });

    statements
}

fn get_label(lambda: &Lambda) -> String {
    format!("lambda_{}", lambda.id.0)
}

/// Returns the list of all lambda expressions in a program.
fn all_lambdas(program: &Program) -> Vec<Lambda> {
    let mut result = vec![];
    for function_definition in &program.function_definitions {
        result.extend(all_lambdas_in_expr(&function_definition.body));
    }
    result.extend(all_lambdas_in_expr(&program.expr));
    result
}

/// Returns the list of all lambda expressions.
fn all_lambdas_in_expr(expr: &Expr) -> Vec<Lambda> {
    match expr {
        Expr::Variable(_) => vec![],
        Expr::Prim1(_, e) => all_lambdas_in_expr(e),
        Expr::Prim2(_, e1, e2) => {
            let mut result = all_lambdas_in_expr(e1);
            result.extend(all_lambdas_in_expr(e2));
            result
        }
        Expr::Prim3(_, e1, e2, e3) => {
            let mut result = all_lambdas_in_expr(e1);
            result.extend(all_lambdas_in_expr(e2));
            result.extend(all_lambdas_in_expr(e3));
            result
        }
        Expr::Begin(e1, e2) => {
            let mut result = all_lambdas_in_expr(e1);
            result.extend(all_lambdas_in_expr(e2));
            result
        }
        Expr::App(App { function, args }) => {
            let mut result = all_lambdas_in_expr(function);
            for arg in args {
                result.extend(all_lambdas_in_expr(arg));
            }
            result
        }
        Expr::If(If { cond, then, els }) => {
            let mut result = all_lambdas_in_expr(cond);
            result.extend(all_lambdas_in_expr(then));
            result.extend(all_lambdas_in_expr(els));
            result
        }
        Expr::Match(Match { expr, arms }) => {
            let mut result = all_lambdas_in_expr(expr);
            for arm in arms {
                result.extend(all_lambdas_in_expr(&arm.body));
            }
            result
        }
        Expr::Let(Let { binding, body }) => {
            let mut result = all_lambdas_in_expr(&binding.rhs);
            result.extend(all_lambdas_in_expr(&body));
            result
        }
        Expr::Lambda(lambda) => vec![lambda.clone()],

        Expr::Lit(_) | Expr::Eof | Expr::Prim0(_) => vec![],
    }
}

/// Returns the set of all free variables in the expression.
fn free_variables(expr: &Expr) -> HashSet<Identifier> {
    match expr {
        Expr::Variable(id) => HashSet::from([id.clone()]),
        Expr::Prim1(_, e) => free_variables(e),
        Expr::Prim2(_, e1, e2) => {
            let mut result = free_variables(e1);
            result.extend(free_variables(e2));
            result
        }
        Expr::Prim3(_, e1, e2, e3) => {
            let mut result = free_variables(e1);
            result.extend(free_variables(e2));
            result.extend(free_variables(e3));
            result
        }
        Expr::Begin(e1, e2) => {
            let mut result = free_variables(e1);
            result.extend(free_variables(e2));
            result
        }
        Expr::App(App { function, args }) => {
            let mut result = free_variables(function);
            for arg in args {
                result.extend(free_variables(arg));
            }
            result
        }
        Expr::If(If { cond, then, els }) => {
            let mut result = free_variables(cond);
            result.extend(free_variables(then));
            result.extend(free_variables(els));
            result
        }
        Expr::Match(Match { expr, arms }) => {
            let mut result = free_variables(expr);
            for arm in arms {
                result.extend(free_variables(&arm.body));
            }
            result
        }
        Expr::Let(Let { binding, body }) => {
            let mut result = free_variables(&binding.rhs);
            result.extend(free_variables(&body));
            result.remove(&binding.lhs);
            result
        }
        Expr::Lambda(Lambda {
            id: _,
            params,
            body,
        }) => {
            let mut result = free_variables(&body);
            for param in params {
                result.remove(param);
            }
            result
        }

        Expr::Lit(_) | Expr::Eof | Expr::Prim0(_) => HashSet::new(),
    }
}
