use compiler_course::mylang;

#[test]
fn it_adds_and_subtracts() {
    let input = "(add1 (sub1 (add1 42)))";
    let result = mylang::run(input).unwrap();
    let expected = "43";
    assert_eq!(result, expected);
}
