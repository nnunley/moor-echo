#[cfg(test)]
mod tests {
    use super::super::{Evaluator, Value};
    use crate::ast::EchoAst;
    use crate::storage::{Storage, ObjectId};
    use std::sync::Arc;

    fn create_test_evaluator() -> (Evaluator, ObjectId) {
        let temp_dir = tempfile::tempdir().unwrap();
        let storage = Arc::new(Storage::new(temp_dir.path()).unwrap());
        let mut evaluator = Evaluator::new(storage);
        let player_id = evaluator.create_player("test_player").unwrap();
        evaluator.switch_player(player_id).unwrap();
        (evaluator, player_id)
    }

    #[test]
    fn test_eval_map_literal() {
        let (mut evaluator, _) = create_test_evaluator();
        
        let map_ast = EchoAst::MapLiteral {
            entries: vec![
                ("name".to_string(), EchoAst::String("Alice".to_string())),
                ("age".to_string(), EchoAst::Number(30)),
                ("active".to_string(), EchoAst::Boolean(true)),
            ],
        };
        
        let result = evaluator.eval(&map_ast).unwrap();
        
        if let Value::Map(map) = result {
            assert_eq!(map.len(), 3);
            assert_eq!(map.get("name"), Some(&Value::String("Alice".to_string())));
            assert_eq!(map.get("age"), Some(&Value::Integer(30)));
            assert_eq!(map.get("active"), Some(&Value::Boolean(true)));
        } else {
            panic!("Expected Map, got: {:?}", result);
        }
    }

    #[test]
    fn test_eval_flyweight() {
        let (mut evaluator, _) = create_test_evaluator();
        
        let flyweight_ast = EchoAst::Flyweight {
            object: Box::new(EchoAst::ObjectRef(1)), // Use root object
            properties: vec![
                ("name".to_string(), EchoAst::String("test_flyweight".to_string())),
                ("value".to_string(), EchoAst::Number(42)),
            ],
        };
        
        let result = evaluator.eval(&flyweight_ast).unwrap();
        
        if let Value::Object(obj_id) = result {
            // Verify the flyweight was created
            let storage = evaluator.storage.clone();
            let obj = storage.objects.get(obj_id).unwrap();
            
            assert_eq!(obj.properties.len(), 2);
            assert!(obj.parent.is_none()); // Flyweights don't inherit
            assert!(obj.verbs.is_empty()); // Flyweights can't have verbs
            
            // Check properties
            use crate::storage::object_store::PropertyValue;
            assert_eq!(obj.properties.get("name"), 
                Some(&PropertyValue::String("test_flyweight".to_string())));
            assert_eq!(obj.properties.get("value"), 
                Some(&PropertyValue::Integer(42)));
        } else {
            panic!("Expected Object, got: {:?}", result);
        }
    }

    #[test]
    fn test_eval_error_catch() {
        let (mut evaluator, _) = create_test_evaluator();
        
        // Test error catching with property access on non-existent object
        let error_catch_ast = EchoAst::ErrorCatch {
            expr: Box::new(EchoAst::PropertyAccess {
                object: Box::new(EchoAst::ObjectRef(999)), // Non-existent object
                property: "nonexistent".to_string(),
            }),
            error_patterns: vec!["Object reference".to_string(), "not found".to_string()],
            default: Box::new(EchoAst::String("default_value".to_string())),
        };
        
        let result = evaluator.eval(&error_catch_ast).unwrap();
        
        assert_eq!(result, Value::String("default_value".to_string()));
    }

    #[test]
    fn test_eval_error_catch_no_match() {
        let (mut evaluator, _) = create_test_evaluator();
        
        // Test error catching where pattern doesn't match
        let error_catch_ast = EchoAst::ErrorCatch {
            expr: Box::new(EchoAst::PropertyAccess {
                object: Box::new(EchoAst::ObjectRef(999)),
                property: "nonexistent".to_string(),
            }),
            error_patterns: vec!["DIFFERENT_ERROR".to_string()],
            default: Box::new(EchoAst::String("should_not_reach".to_string())),
        };
        
        // Should re-throw the error since pattern doesn't match
        let result = evaluator.eval(&error_catch_ast);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_destructuring_assignment() {
        let (mut evaluator, _) = create_test_evaluator();
        
        let destructuring_ast = EchoAst::DestructuringAssignment {
            targets: vec![
                crate::ast::DestructuringTarget::Simple("x".to_string()),
                crate::ast::DestructuringTarget::Simple("y".to_string()),
                crate::ast::DestructuringTarget::Simple("z".to_string()),
            ],
            value: Box::new(EchoAst::List {
                elements: vec![
                    EchoAst::Number(10),
                    EchoAst::Number(20),
                    EchoAst::Number(30),
                ],
            }),
        };
        
        let result = evaluator.eval(&destructuring_ast).unwrap();
        
        // Should return the original list
        if let Value::List(items) = result {
            assert_eq!(items.len(), 3);
            assert_eq!(items[0], Value::Integer(10));
            assert_eq!(items[1], Value::Integer(20));
            assert_eq!(items[2], Value::Integer(30));
        } else {
            panic!("Expected List, got: {:?}", result);
        }
        
        // Verify variables were set
        let x_ast = EchoAst::Identifier("x".to_string());
        let x_val = evaluator.eval(&x_ast).unwrap();
        assert_eq!(x_val, Value::Integer(10));
        
        let y_ast = EchoAst::Identifier("y".to_string());
        let y_val = evaluator.eval(&y_ast).unwrap();
        assert_eq!(y_val, Value::Integer(20));
        
        let z_ast = EchoAst::Identifier("z".to_string());
        let z_val = evaluator.eval(&z_ast).unwrap();
        assert_eq!(z_val, Value::Integer(30));
    }

    #[test]
    fn test_eval_destructuring_assignment_short_list() {
        let (mut evaluator, _) = create_test_evaluator();
        
        // Test destructuring with more targets than list elements
        let destructuring_ast = EchoAst::DestructuringAssignment {
            targets: vec![
                crate::ast::DestructuringTarget::Simple("a".to_string()),
                crate::ast::DestructuringTarget::Simple("b".to_string()),
                crate::ast::DestructuringTarget::Simple("c".to_string()),
            ],
            value: Box::new(EchoAst::List {
                elements: vec![
                    EchoAst::Number(1),
                    EchoAst::Number(2),
                    // Missing third element
                ],
            }),
        };
        
        let _result = evaluator.eval(&destructuring_ast).unwrap();
        
        // Variables beyond list length should be set to null
        let a_val = evaluator.eval(&EchoAst::Identifier("a".to_string())).unwrap();
        let b_val = evaluator.eval(&EchoAst::Identifier("b".to_string())).unwrap();
        let c_val = evaluator.eval(&EchoAst::Identifier("c".to_string())).unwrap();
        
        assert_eq!(a_val, Value::Integer(1));
        assert_eq!(b_val, Value::Integer(2));
        assert_eq!(c_val, Value::Null);
    }

    #[test]
    fn test_eval_spread() {
        let (mut evaluator, _) = create_test_evaluator();
        
        let spread_ast = EchoAst::Spread {
            expr: Box::new(EchoAst::List {
                elements: vec![
                    EchoAst::Number(1),
                    EchoAst::Number(2),
                    EchoAst::Number(3),
                ],
            }),
        };
        
        let result = evaluator.eval(&spread_ast).unwrap();
        
        // For now, spread just returns the evaluated expression
        if let Value::List(items) = result {
            assert_eq!(items.len(), 3);
            assert_eq!(items[0], Value::Integer(1));
            assert_eq!(items[1], Value::Integer(2));
            assert_eq!(items[2], Value::Integer(3));
        } else {
            panic!("Expected List, got: {:?}", result);
        }
    }

    #[test]
    fn test_eval_define_error() {
        let (mut evaluator, _) = create_test_evaluator();
        
        // Define nodes should not be evaluated - they're preprocessor directives
        let define_ast = EchoAst::Define {
            name: "TEST".to_string(),
            value: Box::new(EchoAst::Number(42)),
        };
        
        let result = evaluator.eval(&define_ast);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("preprocessed"));
    }

    #[test]
    fn test_eval_destructuring_non_list() {
        let (mut evaluator, _) = create_test_evaluator();
        
        // Test destructuring with non-list value
        let destructuring_ast = EchoAst::DestructuringAssignment {
            targets: vec![crate::ast::DestructuringTarget::Simple("x".to_string())],
            value: Box::new(EchoAst::String("not a list".to_string())),
        };
        
        let result = evaluator.eval(&destructuring_ast);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("requires a list"));
    }
}