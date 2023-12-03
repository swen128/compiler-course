pub mod a86;
pub mod mylang;

use a86::printer::{CompilationContext, Platform};

fn main() {
    let mylang_source = "(add1 (sub1 (add1 42)))";
    let program = mylang::compile(mylang_source).unwrap();

    let context = CompilationContext {
        platform: Platform::Linux,
    };

    println!("{}", a86::printer::print(&program, &context));
    println!("{}", a86::interpreter::interpret(&program));
}
