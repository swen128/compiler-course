pub mod ast;
pub mod compiler;
pub mod lexer;
pub mod parser;
pub mod s_expression;
mod tokens;
mod document;
mod data_type;

pub fn parse(source: &str) -> Result<ast::Program, String> {
    let tokens = lexer::tokenize(source)?;
    let expr = s_expression::parse(tokens)?;
    parser::parse(&expr)
}
