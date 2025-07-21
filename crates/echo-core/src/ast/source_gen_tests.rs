// Tests for source code generation with Value integration

#[cfg(test)]
mod value_source_tests {
    use crate::evaluator::Value;
    use crate::ast::{EchoAst, LambdaParam};
    use std::collections::HashMap;

    #[test]
    fn test_lambda_value_to_source() {
        // Create a lambda value
        let lambda = Value::Lambda {
            params: vec![
                LambdaParam::Simple("x".to_string()),
                LambdaParam::Simple("y".to_string()),
            ],
            body: EchoAst::Add {
                left: Box::new(EchoAst::Identifier("x".to_string())),
                right: Box::new(EchoAst::Identifier("y".to_string())),
            },
            captured_env: HashMap::new(),
        };
        
        // Get source code from the value
        let source = lambda.to_source();
        assert!(source.is_some());
        
        let source_str = source.unwrap();
        assert!(source_str.contains("fn {x, y}"));
        assert!(source_str.contains("x + y"));
        assert!(source_str.contains("endfn"));
    }
    
    #[test]
    fn test_lambda_with_optional_params_to_source() {
        // Create a lambda with optional parameters
        let lambda = Value::Lambda {
            params: vec![
                LambdaParam::Simple("a".to_string()),
                LambdaParam::Optional {
                    name: "b".to_string(),
                    default: Box::new(EchoAst::Number(10)),
                },
                LambdaParam::Rest("rest".to_string()),
            ],
            body: EchoAst::List {
                elements: vec![
                    EchoAst::Identifier("a".to_string()),
                    EchoAst::Identifier("b".to_string()),
                    EchoAst::Identifier("rest".to_string()),
                ],
            },
            captured_env: HashMap::new(),
        };
        
        let source = lambda.to_source();
        assert!(source.is_some());
        
        let source_str = source.unwrap();
        assert!(source_str.contains("fn {a, ?b = 10, @rest}"));
        assert!(source_str.contains("[a, b, rest]"));
        assert!(source_str.contains("endfn"));
    }
    
    #[test]
    fn test_non_lambda_value_to_source() {
        // Test that non-lambda values return None
        let values = vec![
            Value::Null,
            Value::Boolean(true),
            Value::Integer(42),
            Value::Float(3.14),
            Value::String("test".to_string()),
            Value::List(vec![Value::Integer(1), Value::Integer(2)]),
        ];
        
        for value in values {
            assert_eq!(value.to_source(), None);
        }
    }
}