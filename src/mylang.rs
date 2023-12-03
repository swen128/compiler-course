pub mod ast;
pub mod compiler;
pub mod lexer;
pub mod parser;
mod s_expression;
mod tokens;
mod document;

pub fn parse(source: &str) -> Result<ast::Program, String> {
    let tokens = lexer::tokenize(source)?;
    let expr = s_expression::parse(tokens)?;
    parser::parse(expr)
}

pub fn compile(source: &str) -> Result<crate::a86::ast::Program, String> {
    parse(source).map(|mylang_program| compiler::compile(mylang_program))
}
