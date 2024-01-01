use crate::a86::ast::Statement;

use super::external_call::compile_raise_error;

/// Jump to this label to raise an error.
pub const ERR_LABEL: &str = "err";

/// Put these instructions once at the end of the program.
/// Jump to [ERR_LABEL] whenever you want to raise an error.
pub fn compile_error_handler() -> Vec<Statement> {
    let mut statements = vec![Statement::Label {
        name: ERR_LABEL.to_string(),
    }];
    statements.extend(compile_raise_error());
    statements
}
