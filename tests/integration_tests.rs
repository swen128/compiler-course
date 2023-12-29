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

        Err(Error::ParserError(ParserError::AstPasringError(err))) => {
            assert_eq!(err.position, Position::new(1));
        }

        Err(err) => {
            panic!("Expected a AST parsing error. Got: {:?}", err);
        }
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

#[test]
fn write_byte() {
    let input = "(write-byte 97)";
    let result = run(input).unwrap();
    let expected = "a";
    assert_eq!(result, expected);
}

#[test]
fn begin() {
    let input = "(begin (write-byte 97) (write-byte 98))";
    let result = run(input).unwrap();
    let expected = "ab";
    assert_eq!(result, expected);
}

#[test]
fn read_void() {
    let input = "(read-byte)";
    let result = run(input).unwrap();
    let expected = "";
    assert_eq!(result, expected);
}

#[test]
fn echo_back() {
    let source = "(write-byte (read-byte))";
    let input = "abc";
    let result = run_with_stdin(source, input).unwrap();
    let expected = "a";
    assert_eq!(result, expected);
}

#[test]
fn peek_byte() {
    let source = "(begin (write-byte (peek-byte)) (write-byte (peek-byte)))";
    let input = "abc";
    let result = run_with_stdin(source, input).unwrap();
    let expected = "aa";
    assert_eq!(result, expected);
}

#[test]
fn add_invalid_type() {
    let input = "(add1 #\\a)";
    let result = run(input).unwrap_err();
    assert_eq!(result, Error::RuntimeError);
}

#[test]
fn sub_invalid_type() {
    let input = "(sub1 #\\a)";
    let result = run(input).unwrap_err();
    assert_eq!(result, Error::RuntimeError);
}

#[test]
fn invalid_codepoint_to_char() {
    let input = "(integer->char 99999999)";
    let result = run(input).unwrap_err();
    assert_eq!(result, Error::RuntimeError);
}

#[test]
fn let_expression() {
    let input = "(let ((x 42)) x)";
    let result = run(input).unwrap();
    let expected = "42";
    assert_eq!(result, expected);
}

#[test]
fn two_variables() {
    let input = "(let ((x 42)) (let ((y 43)) x))";
    let result = run(input).unwrap();
    let expected = "42";
    assert_eq!(result, expected);
}

#[test]
fn write_two_variables() {
    let input = "(let ((x 97)) (let ((y 98)) (begin (write-byte x) (write-byte y))))";
    let result = run(input).unwrap();
    let expected = "ab";
    assert_eq!(result, expected);
}

#[test]
fn add_two_variables() {
    let input = "(let ((x 42)) (let ((y 1)) (+ x y)))";
    let result = run(input).unwrap();
    let expected = "43";
    assert_eq!(result, expected);
}

fn run_with_stdin(source: &str, input: &str) -> Result<String, Error> {
    use std::io::Write;
    use std::process::{Command, Stdio};

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

    let mut command = Command::new(&bin);
    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to execute process");

    child
        .stdin
        .as_mut()
        .take()
        .expect("Failed to open stdin")
        .write_all(input.as_bytes())
        .expect("Failed to write to stdin");

    let output = child.wait_with_output().expect("failed to execute process");

    if output.status.code() == Some(1) {
        return Err(Error::RuntimeError);
    }

    if !output.status.success() {
        panic!("process failed with the output: {:?}", output);
    }

    let stdout = String::from_utf8(output.stdout).expect("invalid utf8");
    Ok(stdout)
}

fn run(source: &str) -> Result<String, Error> {
    run_with_stdin(source, "")
}

fn hash_str(str: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    str.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

#[derive(Debug, PartialEq)]
enum Error {
    ParserError(ParserError),
    RuntimeError,
}

impl From<ParserError> for Error {
    fn from(err: ParserError) -> Self {
        Error::ParserError(err)
    }
}
