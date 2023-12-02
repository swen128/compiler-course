use crate::a86::ast::*;
use crate::a86::printer::*;
use std::process::Command;

pub fn interpret(program: &Program) -> String {
    let asm = print(
        program,
        &CompilationContext {
            platform: Platform::Linux,
        },
    );

    std::fs::create_dir_all("out").expect("failed to create directory");
    
    std::fs::write("out/asm.s", asm).expect("failed to write file");

    Command::new("nasm")
        .args(&["-f", "elf64", "-o", "out/asm.o", "out/asm.s"])
        .output()
        .expect("failed to execute process");

    Command::new("gcc")
        .args(&["-o", "out/main.o", "-c", "src/a86/main.c"])
        .output()
        .expect("failed to execute process");

    Command::new("gcc")
        .args(&["-o", "out/output", "out/main.o", "out/asm.o"])
        .output()
        .expect("failed to execute process");

    // Print the command output
    let mut command = std::process::Command::new("./out/output");
    let output = command.output().expect("failed to execute process");
    let stdout = String::from_utf8(output.stdout).unwrap();
    stdout
}
