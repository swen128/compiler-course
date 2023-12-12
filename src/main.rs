use compiler_course::compile;

fn main() {
    let mylang_source = "(add1 (sub1 (add1 42)))";
    let program = compile(mylang_source).unwrap();
    println!("{}", program);
}
