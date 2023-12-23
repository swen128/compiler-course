mod a86;
mod mylang;

pub use mylang::parse;
pub use mylang::ParserError;
pub use mylang::document::Position;

pub fn compile(source: &str) -> Result<String, ParserError> {
    parse(source)
        .map(|ast| mylang::compiler::compile(ast))
        .map(|a86_program| {
            let context = a86::printer::CompilationContext {
                platform: a86::printer::Platform::Linux,
            };
            a86::printer::print(&a86_program, &context)
        })
}
