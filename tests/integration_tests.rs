use compiler_course::mylang;

#[test]
fn negative_number() {
    let input = "-42";
    let result = mylang::run(input).unwrap();
    let expected = "-42";
    assert_eq!(result, expected);
}

#[test]
fn it_adds_and_subtracts() {
    let input = "(add1 (sub1 (add1 42)))";
    let result = mylang::run(input).unwrap();
    let expected = "43";
    assert_eq!(result, expected);
}

#[test]
fn invalid_syntax() {
    let input = "((add1 (sub1 (add1 42))))";
    let result = mylang::run(input);
    
    assert_eq!(result, Err("Expected operator".to_string()));
}

#[test]
fn if_zero() {
    let input = "(if (zero? 0) 42 43)";
    let result = mylang::run(input).unwrap();
    let expected = "42";
    assert_eq!(result, expected);
}

#[test]
fn if_nonzero() {
    let input = "(if (zero? 1) 42 43)";
    let result = mylang::run(input).unwrap();
    let expected = "43";
    assert_eq!(result, expected);
}

#[test]
fn nested_if() {
    let input = "(add1 (if (zero? (if (zero? 1) 0 43)) -21 18))";
    let result = mylang::run(input).unwrap();
    let expected = "19";
    assert_eq!(result, expected);
}
