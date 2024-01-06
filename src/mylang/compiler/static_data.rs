use crate::a86::ast::Statement;

use super::{state::Compiler, string::compile_all_string_data};

pub fn compile_data_section(compiler: &Compiler) -> Vec<Statement> {
    let mut statements = vec![Statement::Data];
    statements.extend(compile_all_string_data(compiler));
    statements
}
