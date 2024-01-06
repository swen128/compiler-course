use crate::a86::ast::{Operand, Program, Register, Statement};

use self::{
    error::compile_error_handler,
    expr::compile_expr,
    external_call::externals,
    function::{
        compile_closures_for_defines, compile_defines, compile_lambda_definitions, defined_ids,
    },
    state::Compiler,
    string::all_string_literals,
    variable::VariablesTable, static_data::compile_data_section,
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
mod pattern_match;
mod primitive_functions;
mod state;
mod string;
mod types;
mod variable;
mod vector;
mod static_data;

const RBX: Operand = Operand::Register(Register::RBX);
const RDI: Operand = Operand::Register(Register::RDI);
const RSP: Operand = Operand::Register(Register::RSP);
const R15: Operand = Operand::Register(Register::R15);

pub fn compile(program: ast::Program) -> Program {
    let string_literals = all_string_literals(&program);
    let mut compiler = Compiler::new(string_literals);

    let mut statements = vec![Statement::Global {
        name: "entry".to_string(),
    }];
    statements.extend(externals());
    statements.push(Statement::Label {
        name: "entry".to_string(),
    });

    // Stash callee-saved registers.
    statements.push(Statement::Push { src: RBX });
    statements.push(Statement::Push { src: R15 });

    statements.push(Statement::Mov {
        dest: RBX,
        src: RDI, // The runtime must allocate the heap memory and pass its address via rdi.
    });

    statements.extend(compile_closures_for_defines(&program));
    let env = VariablesTable::new().extended(defined_ids(&program));

    statements.extend(compile_expr(
        program.expr.clone(),
        &mut compiler,
        &env,
        false,
    ));

    // Pop function definitions
    statements.push(Statement::Add {
        dest: RSP,
        src: Operand::Immediate(8 * program.function_definitions.len() as i64),
    });

    // Restore callee-saved registers.
    statements.push(Statement::Pop { dest: R15 });
    statements.push(Statement::Pop { dest: RBX });

    statements.push(Statement::Ret);

    statements.extend(compile_defines(&program, &mut compiler));
    statements.extend(compile_lambda_definitions(&program, &mut compiler));

    statements.extend(compile_error_handler());
    statements.extend(compile_data_section(&compiler));

    Program { statements }
}
