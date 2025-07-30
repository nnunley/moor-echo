// Tests for simple parser functionality
// Following TDD - write tests first, then refactor

#[cfg(test)]
mod tests {
    use crate::ast::{BindingPattern, BindingType, EchoAst, LValue, ObjectMember};

    use crate::parser::simple_parser::parse_simple;

    #[test]
    fn test_parse_integer() {
        let result = parse_simple("42");
        assert_eq!(result, Ok(EchoAst::Number(42)));
    }

    #[test]
    fn test_parse_string() {
        let result = parse_simple("\"hello world\"");
        assert_eq!(result, Ok(EchoAst::String("hello world".to_string())));
    }

    #[test]
    fn test_parse_identifier() {
        let result = parse_simple("myVariable");
        assert_eq!(result, Ok(EchoAst::Identifier("myVariable".to_string())));
    }

    #[test]
    fn test_parse_let_statement() {
        let result = parse_simple("let x = 42;");
        assert_eq!(
            result,
            Ok(EchoAst::Assignment {
                target: LValue::Binding {
                    binding_type: BindingType::Let,
                    pattern: BindingPattern::Identifier("x".to_string()),
                },
                value: Box::new(EchoAst::Number(42)),
            })
        );
    }

    #[test]
    fn test_parse_let_with_string() {
        let result = parse_simple("let name = \"Echo\";");
        assert_eq!(
            result,
            Ok(EchoAst::Assignment {
                target: LValue::Binding {
                    binding_type: BindingType::Let,
                    pattern: BindingPattern::Identifier("name".to_string()),
                },
                value: Box::new(EchoAst::String("Echo".to_string())),
            })
        );
    }

    #[test]
    fn test_parse_property_assignment() {
        let result = parse_simple("obj.prop = 100;");
        assert_eq!(
            result,
            Ok(EchoAst::Assignment {
                target: LValue::PropertyAccess {
                    object: Box::new(EchoAst::Identifier("obj".to_string())),
                    property: "prop".to_string(),
                },
                value: Box::new(EchoAst::Number(100)),
            })
        );
    }

    #[test]
    fn test_parse_system_property_assignment() {
        let result = parse_simple("$system.name = \"test\";");
        assert_eq!(
            result,
            Ok(EchoAst::Assignment {
                target: LValue::PropertyAccess {
                    object: Box::new(EchoAst::SystemProperty("system".to_string())),
                    property: "name".to_string(),
                },
                value: Box::new(EchoAst::String("test".to_string())),
            })
        );
    }

    #[test]
    fn test_parse_property_access() {
        let result = parse_simple("player.name");
        assert_eq!(
            result,
            Ok(EchoAst::PropertyAccess {
                object: Box::new(EchoAst::Identifier("player".to_string())),
                property: "name".to_string(),
            })
        );
    }

    #[test]
    fn test_parse_binary_operation() {
        let result = parse_simple("10 + 20");
        assert_eq!(
            result,
            Ok(EchoAst::Add {
                left: Box::new(EchoAst::Number(10)),
                right: Box::new(EchoAst::Number(20)),
            })
        );
    }

    #[test]
    fn test_parse_property_with_binary_op() {
        let result = parse_simple("obj.value + 10");
        assert_eq!(
            result,
            Ok(EchoAst::Add {
                left: Box::new(EchoAst::PropertyAccess {
                    object: Box::new(EchoAst::Identifier("obj".to_string())),
                    property: "value".to_string(),
                }),
                right: Box::new(EchoAst::Number(10)),
            })
        );
    }

    #[test]
    fn test_parse_method_call() {
        let result = parse_simple("obj:method()");
        assert_eq!(
            result,
            Ok(EchoAst::MethodCall {
                object: Box::new(EchoAst::Identifier("obj".to_string())),
                method: "method".to_string(),
                args: vec![],
            })
        );
    }

    #[test]
    fn test_parse_method_call_with_args() {
        let result = parse_simple("player:move(10, 20)");
        assert_eq!(
            result,
            Ok(EchoAst::MethodCall {
                object: Box::new(EchoAst::Identifier("player".to_string())),
                method: "move".to_string(),
                args: vec![EchoAst::Number(10), EchoAst::Number(20)],
            })
        );
    }

    #[test]
    fn test_parse_object_definition() {
        let input = r#"
object MyObject extends BaseObject
    property name = "test"
    property value = 42
endobject
"#;
        let result = parse_simple(input);
        assert_eq!(
            result,
            Ok(EchoAst::ObjectDef {
                name: "MyObject".to_string(),
                parent: Some("BaseObject".to_string()),
                members: vec![
                    ObjectMember::Property {
                        name: "name".to_string(),
                        value: EchoAst::String("test".to_string()),
                        permissions: None,
                    },
                    ObjectMember::Property {
                        name: "value".to_string(),
                        value: EchoAst::Number(42),
                        permissions: None,
                    },
                ],
            })
        );
    }

    #[test]
    fn test_parse_object_with_verb() {
        let input = r#"
object Player
    property name = "player"
    verb "greet" ()
        return "Hello!";
    endverb
endobject
"#;
        let result = parse_simple(input);
        match result {
            Ok(EchoAst::ObjectDef { name, members, .. }) => {
                assert_eq!(name, "Player");
                assert_eq!(members.len(), 2);
                match &members[1] {
                    ObjectMember::Verb { name, body, .. } => {
                        assert_eq!(name, "greet");
                        assert!(!body.is_empty());
                    }
                    _ => panic!("Expected verb member"),
                }
            }
            _ => panic!("Expected ObjectDef"),
        }
    }

    #[test]
    fn test_parse_boolean_values() {
        assert_eq!(parse_simple("true"), Ok(EchoAst::Boolean(true)));
        assert_eq!(parse_simple("false"), Ok(EchoAst::Boolean(false)));
    }

    #[test]
    fn test_parse_null() {
        assert_eq!(parse_simple("null"), Ok(EchoAst::Null));
    }

    #[test]
    fn test_parse_errors() {
        // Test various invalid inputs
        assert!(parse_simple("").is_err());
        assert!(parse_simple("let = 5").is_err());
        assert!(parse_simple("obj:").is_err());
        assert!(parse_simple("obj:method(").is_err());
    }
}