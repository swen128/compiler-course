use compiler_course::compile;

#[test]
fn negative_number() {
    let input = "-42";
    let result = run(input).unwrap();
    let expected = "-42";
    assert_eq!(result, expected);
}

#[test]
fn it_adds_and_subtracts() {
    let input = "(add1 (sub1 (add1 42)))";
    let result = run(input).unwrap();
    let expected = "43";
    assert_eq!(result, expected);
}

#[test]
fn invalid_syntax() {
    let input = "((add1 (sub1 (add1 42))))";
    let result = run(input);

    assert_eq!(result, Err("Expected operator".to_string()));
}

#[test]
fn if_zero() {
    let input = "(if (zero? 0) 42 43)";
    let result = run(input).unwrap();
    let expected = "42";
    assert_eq!(result, expected);
}

#[test]
fn if_nonzero() {
    let input = "(if (zero? 1) 42 43)";
    let result = run(input).unwrap();
    let expected = "43";
    assert_eq!(result, expected);
}

#[test]
fn nested_if() {
    let input = "(add1 (if (zero? (if (zero? 1) 0 43)) -21 18))";
    let result = run(input).unwrap();
    let expected = "19";
    assert_eq!(result, expected);
}

#[test]
fn if_false() {
    let input = "(if #f 42 43)";
    let result = run(input).unwrap();
    let expected = "43";
    assert_eq!(result, expected);
}

#[test]
fn if_true() {
    let input = "(if #t 42 43)";
    let result = run(input).unwrap();
    let expected = "42";
    assert_eq!(result, expected);
}

#[test]
fn if_non_boolean() {
    let input = "(if -1 42 43)";
    let result = run(input).unwrap();
    let expected = "42";
    assert_eq!(result, expected);
}

pub fn run(source: &str) -> Result<String, String> {
    use std::process::Command;

    let asm = compile(source)?;

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

    let mut command = std::process::Command::new(&bin);
    let output = command.output().expect("failed to execute process");
    let stdout = String::from_utf8(output.stdout).expect("invalid utf8");
    Ok(stdout)
}

fn hash_str(str: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    str.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}
