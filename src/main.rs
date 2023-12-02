mod a86;

use a86::Statement::*;
use a86::Operand::*;
use a86::Register::*;

fn main() {
    let program = a86::Program {
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
    
    let context = a86::CompilationContext {
        platform: a86::Platform::Linux,
    };
    
    println!("{}", a86::print(program, context));
}
