use std::error::Error;

use compiler_course::{compile, ParserError, Position};

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
    
    match result {
        Ok(_) => panic!("Expected a parser error."),
        
        Err(ParserError::AstPasringError(err)) => {
            assert_eq!(err.position, Position::new(0, 1));
        },

        Err(err) => {
            panic!("Expected a AST parsing error. Got: {:?}", err);
        },
    }
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

#[test]
fn is_char() {
    let input = "(char? #\\a)";
    let result = run(input).unwrap();
    let expected = "#t";
    assert_eq!(result, expected);
}

#[test]
fn is_not_char() {
    let input = "(char? 42)";
    let result = run(input).unwrap();
    let expected = "#f";
    assert_eq!(result, expected);
}

#[test]
fn int_to_char() {
    let input = "(integer->char 97)";
    let result = run(input).unwrap();
    let expected = "#\\a";
    assert_eq!(result, expected);
}

#[test]
fn char_to_int() {
    let input = "(char->integer #\\a)";
    let result = run(input).unwrap();
    let expected = "97";
    assert_eq!(result, expected);
}

pub fn run(source: &str) -> Result<String, ParserError> {
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
