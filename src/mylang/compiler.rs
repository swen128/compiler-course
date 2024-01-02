use crate::a86::ast::{Operand, Program, Register, Statement};

use self::{
    error::compile_error_handler, expr::compile_expr, external_call::externals, state::Compiler,
    variable::VariablesTable,
};

use super::ast;

mod arithmetic;
mod box_type;
mod cons;
mod error;
mod expr;
mod external_call;
mod function;
mod helper;
mod primitive_functions;
mod state;
mod string;
mod types;
mod variable;
mod vector;

const RBX: Operand = Operand::Register(Register::RBX);
const RDI: Operand = Operand::Register(Register::RDI);

pub fn compile(program: ast::Program) -> Program {
    let mut compiler = Compiler::new();
    let env = VariablesTable::new();
    let mut statements = vec![Statement::Global {
        name: "entry".to_string(),
    }];
    statements.extend(externals());
    statements.push(Statement::Label {
        name: "entry".to_string(),
    });
    statements.push(Statement::Mov {
        dest: RBX,
        src: RDI, // The runtime must allocate the heap memory and pass its address via rdi.
    });
    statements.extend(compile_expr(program.expr, &mut compiler, &env));
    statements.push(Statement::Ret);
    statements.extend(compile_error_handler());

    for function_definition in program.function_definitions {
        statements.extend(function::compile_function_definition(
            function_definition,
            &mut compiler,
            &env,
        ));
    }

    Program { statements }
}
