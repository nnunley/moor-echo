use echo_repl::parser::grammar::{parse_echo, EchoAst};

#[test]
fn test_rust_sitter_number() {
    let result = parse_echo("42");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), EchoAst::Number(42));
}

#[test]
fn test_rust_sitter_identifier() {
    let result = parse_echo("hello");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), EchoAst::Identifier("hello".to_string()));
}

#[test]
fn test_rust_sitter_addition() {
    let result = parse_echo("1 + 2");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), EchoAst::Add {
        left: Box::new(EchoAst::Number(1)),
        _op: (),
        right: Box::new(EchoAst::Number(2))
    });
}

#[test]
fn test_rust_sitter_property_access() {
    let result = parse_echo("obj.prop");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), EchoAst::PropertyAccess {
        object: Box::new(EchoAst::Identifier("obj".to_string())),
        _dot: (),
        property: Box::new(EchoAst::Identifier("prop".to_string()))
    });
}