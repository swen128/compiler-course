pub mod a86;

use a86::ast::{Operand::*, Program, Register::*, Statement::*};

use a86::printer::{CompilationContext, Platform};

fn main() {
    let program = tri(36);

    let context = CompilationContext {
        platform: Platform::Linux,
    };

    println!("{}", a86::printer::print(&program, &context));
    println!("{}", a86::interpreter::interpret(&program));
}

fn tri(n: i64) -> Program {
    let statements = vec![
        Global {
            name: "entry".to_string(),
        },
        Label {
            name: "entry".to_string(),
        },
        Mov {
            dest: Register(RBX),
            src: Immediate(n),
        },
        Label {
            name: "tri".to_string(),
        },
        Cmp {
            dest: Register(RBX),
            src: Immediate(0),
        },
        Je {
            label: "end".to_string(),
        },
        Push { src: Register(RBX) },
        Sub {
            dest: Register(RBX),
            src: Immediate(1),
        },
        Call {
            label: "tri".to_string(),
        },
        Pop {
            dest: Register(RBX),
        },
        Add {
            dest: Register(RAX),
            src: Register(RBX),
        },
        Ret,
        Label {
            name: "end".to_string(),
        },
        Mov {
            dest: Register(RAX),
            src: Immediate(0),
        },
        Ret,
    ];

    Program { statements }
}
