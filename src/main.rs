pub mod a86;

use a86::ast::{
    Statement::*,
    Operand::*,
    Register::*,
};

use a86::printer::{
    CompilationContext,
    Platform,
};

fn main() {
    let program = a86::ast::Program {
        statements: vec![
            Global { name: "main".to_string() },
            Label { name: "main".to_string() },
            Mov {
                dest: Register(RAX),
                src: Immediate(0),
            },
            Ret,
        ],
    };
    
    let context = CompilationContext {
        platform: Platform::Linux,
    };
    
    println!("{}", a86::printer::print(program, context));
}
