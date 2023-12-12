use super::ast::*;
use super::printer::*;
use std::process::Command;

pub fn interpret(program: &Program) -> String {
    let asm = print(
        program,
        &CompilationContext {
            platform: Platform::Linux,
        },
    );

    let asm_filename = format!("out/{}.asm", hash_str(&asm));
    let asm_output = format!("out/{}.o", hash_str(&asm));
    let bin = format!("out/{}.run", hash_str(&asm));

    std::fs::write(&asm_filename, asm).expect("failed to write file");

    Command::new("nasm")
        .args(&["-f", "elf64", "-o", &asm_output, &asm_filename])
        .output()
        .expect("failed to execute process");

    Command::new("make")
        .output()
        .expect("failed to execute process");

    Command::new("gcc")
        .args(&["-o", &bin, "out/runtime.o", &asm_output])
        .output()
        .expect("failed to execute process");

    // Print the command output
    let mut command = std::process::Command::new(&bin);
    let output = command.output().expect("failed to execute process");
    let stdout = String::from_utf8(output.stdout).unwrap();
    stdout
}

fn hash_str(str: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    str.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}
