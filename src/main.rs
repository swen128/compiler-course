use std::io::Read;

use compiler_course::compile;

/// Reads source code from stdin and prints the compiled assembly to stdout.
fn main() {
    let mut source = String::new();
    std::io::stdin().read_to_string(&mut source).unwrap();

    let program = compile(&source).unwrap();
    println!("{}", program);
}
