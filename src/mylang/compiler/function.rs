// Function calling convention:
// * The caller pushes the return address and the arguments to the stack before calling function.
// * The last argument is at the top of the stack.
// * The return address is right below all the arguments.
// * The callee pops all the arguments before returning.

use super::{expr::compile_expr, state::Compiler, variable::VariablesTable};
use crate::{
    a86::ast::{Operand, Register, Statement},
    mylang::ast::{App, FunctionDefinition},
};

const RAX: Operand = Operand::Register(Register::RAX);
const R8: Operand = Operand::Register(Register::R8);

/// Returns labled instructions defining the function.
/// It should be put after the main program and invoked by [compile_function_application].
pub fn compile_function_definition(
    function_definition: FunctionDefinition,
    compiler: &mut Compiler,
    env: &VariablesTable,
) -> Vec<Statement> {
    let function_name = function_definition.signature.name.0;
    let body = function_definition.body;
    let params = function_definition.signature.params;
    let n_params = params.len();

    let mut statements = vec![Statement::Label {
        name: function_name,
    }];

    let new_env = env.extended(params);
    statements.extend(compile_expr(body, compiler, &new_env, true));

    // Pop the arguments from the stack and discard them.
    statements.push(Statement::Add {
        dest: Operand::Register(Register::RSP),
        src: Operand::Immediate((n_params * 8) as i64),
    });

    statements.push(Statement::Ret);

    statements
}

/// Returns instrcutions to invoke a function defined by [compile_function_definition].
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

    let function_name = function_application.function.0;
    let args = function_application.args;

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

    // And then push the arguments to the stack.
    for arg in args {
        statements.extend(compile_expr(arg, compiler, &env, false));
        statements.push(Statement::Push { src: RAX });
        env = env.with_non_var();
    }

    statements.push(Statement::Jmp {
        label: function_name,
    });
    statements.push(Statement::Label { name: return_label });

    statements
}

/// Returns optimized instructions for tail calls.
fn compile_function_application_tail(
    function_application: App,
    compiler: &mut Compiler,
    env: &VariablesTable,
) -> Vec<Statement> {
    let function_name = function_application.function.0;
    let args = function_application.args;
    let n_args = args.len();
    let n_env = env.len();

    let mut statements = vec![];

    let mut new_env = env.clone();
    for arg in args {
        statements.extend(compile_expr(arg, compiler, &new_env, false));
        statements.push(Statement::Push { src: RAX });
        new_env = new_env.with_non_var();
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
    //         |       3              |
    //         +----------------------+
    // rsp --> |       5              |
    //         +----------------------+
    //
    // The `move_args` function does just that.
    statements.extend(move_args(n_args, n_env));

    statements.push(Statement::Jmp {
        label: function_name,
    });

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
