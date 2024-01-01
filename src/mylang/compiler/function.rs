// Function calling convention:
// * The caller pushes the return address and the arguments to the stack before calling function.
// * The last argument is at the top of the stack.
// * The return address is right below all the arguments.
// * The callee pops all the arguments before returning.

use super::{expr::compile_expr, state::Compiler};
use crate::{
    a86::ast::{Operand, Register, Statement},
    mylang::ast::{App, FunctionDefinition},
};

const RAX: Operand = Operand::Register(Register::RAX);

/// Returns labled instructions defining the function.
/// It should be put after the main program and invoked by [compile_function_application].
pub fn compile_function_definition(
    function_definition: FunctionDefinition,
    compiler: &mut Compiler,
) -> Vec<Statement> {
    let function_name = function_definition.signature.name.0;
    let body = function_definition.body;
    let params = function_definition.signature.params;
    let n_params = params.len();

    for param in params {
        compiler.variables_table.push_variable(param);
    }

    let mut statements = vec![Statement::Label {
        name: function_name,
    }];

    statements.extend(compile_expr(body, compiler));

    // Pop the arguments from the stack and discard them.
    statements.push(Statement::Add {
        dest: Operand::Register(Register::RSP),
        src: Operand::Immediate((n_params * 8) as i64),
    });

    statements.push(Statement::Ret);

    for _ in 0..n_params {
        compiler.variables_table.pop();
    }

    statements
}

/// Returns instrcutions to invoke a function defined by [compile_function_definition].
pub fn compile_function_application(
    function_application: App,
    compiler: &mut Compiler,
) -> Vec<Statement> {
    let return_label = format!("function_return_site_{}", compiler.new_label_id());

    let function_name = function_application.function.0;
    let args = function_application.args;
    let n_args = args.len();

    let mut statements = vec![
        Statement::Lea {
            dest: RAX,
            label: return_label.clone(),
        },
        Statement::Push { src: RAX },
    ];
    compiler.variables_table.push_non_variable();

    for arg in args {
        statements.extend(compile_expr(arg, compiler));
        statements.push(Statement::Push { src: RAX });
        compiler.variables_table.push_non_variable();
    }

    statements.push(Statement::Jmp {
        label: function_name,
    });
    statements.push(Statement::Label { name: return_label });

    for _ in 0..n_args {
        compiler.variables_table.pop();
    }

    statements
}
