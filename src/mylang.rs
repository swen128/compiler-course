pub mod ast;
pub mod compiler;
mod data_type;
pub mod document;
mod error;
pub mod lexer;
pub mod parser;
pub mod s_expression;

pub use error::ParserError;

pub fn parse(source: &str) -> Result<ast::Program, ParserError> {
    let tokens = lexer::tokenize(source)?;
    let s_expressions = s_expression::parse(tokens)?;
    parser::parse(&s_expressions).or_else(|err| Err(ParserError::from(err)))
}
