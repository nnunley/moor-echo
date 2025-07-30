//! Tests for JIT compilation of access operations

#[cfg(all(test, feature = "jit"))]
mod tests {
    use super::super::*;
    use crate::ast::{EchoAst, LValue, BindingType, BindingPattern};
    use crate::storage::Storage;
    use std::sync::Arc;
    use tempfile::TempDir;

    fn create_test_jit() -> JitEvaluator {
        let temp_dir = TempDir::new().unwrap();
        let storage = Arc::new(Storage::new(temp_dir.path()).unwrap());
        JitEvaluator::new_with_fallback(storage)
    }

    #[test]
    fn test_jit_index_access() {
        let mut jit = create_test_jit();
        
        // Create a player
        let player_id = jit.create_player("test_player").unwrap();
        jit.switch_player(player_id).unwrap();
        
        // Test list index access
        let list_ast = EchoAst::List {
            elements: vec![
                EchoAst::Number(10),
                EchoAst::Number(20),
                EchoAst::Number(30),
            ],
        };
        
        // Store list in a variable
        let assign_ast = EchoAst::Assignment {
            target: LValue::Binding {
                binding_type: BindingType::Let,
                pattern: BindingPattern::Identifier("mylist".to_string()),
            },
            value: Box::new(list_ast),
        };
        jit.eval(&assign_ast).unwrap();
        
        // Access list[1]
        let ast = EchoAst::IndexAccess {
            object: Box::new(EchoAst::Identifier("mylist".to_string())),
            index: Box::new(EchoAst::Number(1)),
        };
        let result = jit.eval(&ast).unwrap();
        
        match result {
            Value::Integer(n) => assert_eq!(n, 20),
            _ => panic!("Expected integer value"),
        }
    }

    #[test]
    fn test_jit_map_index_access() {
        let mut jit = create_test_jit();
        
        // Create a player
        let player_id = jit.create_player("test_player").unwrap();
        jit.switch_player(player_id).unwrap();
        
        // Create and store a map
        let map_ast = EchoAst::Map {
            entries: vec![
                ("name".to_string(), EchoAst::String("Alice".to_string())),
                ("age".to_string(), EchoAst::Number(30)),
            ],
        };
        
        let assign_ast = EchoAst::Assignment {
            target: LValue::Binding {
                binding_type: BindingType::Let,
                pattern: BindingPattern::Identifier("person".to_string()),
            },
            value: Box::new(map_ast),
        };
        jit.eval(&assign_ast).unwrap();
        
        // Access map["name"]
        let ast = EchoAst::IndexAccess {
            object: Box::new(EchoAst::Identifier("person".to_string())),
            index: Box::new(EchoAst::String("name".to_string())),
        };
        let result = jit.eval(&ast).unwrap();
        
        match result {
            Value::String(s) => assert_eq!(s, "Alice"),
            _ => panic!("Expected string value"),
        }
        
        // Access map["age"]
        let ast = EchoAst::IndexAccess {
            object: Box::new(EchoAst::Identifier("person".to_string())),
            index: Box::new(EchoAst::String("age".to_string())),
        };
        let result = jit.eval(&ast).unwrap();
        
        match result {
            Value::Integer(n) => assert_eq!(n, 30),
            _ => panic!("Expected integer value"),
        }
    }

    #[test]
    fn test_jit_property_access() {
        let mut jit = create_test_jit();
        
        // Create a player
        let player_id = jit.create_player("test_player").unwrap();
        jit.switch_player(player_id).unwrap();
        
        // PropertyAccess typically works with objects, not maps
        // For now, test with a map and string property access
        let map_ast = EchoAst::Map {
            entries: vec![
                ("x".to_string(), EchoAst::Number(42)),
                ("y".to_string(), EchoAst::Number(99)),
            ],
        };
        
        let assign_ast = EchoAst::Assignment {
            target: LValue::Binding {
                binding_type: BindingType::Let,
                pattern: BindingPattern::Identifier("point".to_string()),
            },
            value: Box::new(map_ast),
        };
        jit.eval(&assign_ast).unwrap();
        
        // Access point.x
        let ast = EchoAst::PropertyAccess {
            object: Box::new(EchoAst::Identifier("point".to_string())),
            property: "x".to_string(),
        };
        let result = jit.eval(&ast).unwrap();
        
        match result {
            Value::Integer(n) => assert_eq!(n, 42),
            _ => panic!("Expected integer value"),
        }
    }

    #[test]
    fn test_jit_nested_access() {
        let mut jit = create_test_jit();
        
        // Create a player
        let player_id = jit.create_player("test_player").unwrap();
        jit.switch_player(player_id).unwrap();
        
        // Create nested structure
        let inner_map = EchoAst::Map {
            entries: vec![
                ("value".to_string(), EchoAst::Number(123)),
            ],
        };
        
        let outer_list = EchoAst::List {
            elements: vec![
                inner_map,
                EchoAst::Number(456),
            ],
        };
        
        let assign_ast = EchoAst::Assignment {
            target: LValue::Binding {
                binding_type: BindingType::Let,
                pattern: BindingPattern::Identifier("data".to_string()),
            },
            value: Box::new(outer_list),
        };
        jit.eval(&assign_ast).unwrap();
        
        // Access data[0]["value"]
        let ast = EchoAst::IndexAccess {
            object: Box::new(EchoAst::IndexAccess {
                object: Box::new(EchoAst::Identifier("data".to_string())),
                index: Box::new(EchoAst::Number(0)),
            }),
            index: Box::new(EchoAst::String("value".to_string())),
        };
        let result = jit.eval(&ast).unwrap();
        
        match result {
            Value::Integer(n) => assert_eq!(n, 123),
            _ => panic!("Expected integer value"),
        }
    }

    #[test]
    fn test_jit_compile_access() {
        let mut jit = create_test_jit();
        
        // Skip test if JIT is not enabled
        if !jit.is_jit_enabled() {
            println!("Skipping access compilation test - JIT not enabled");
            return;
        }
        
        // Test that compile_ast handles access operations
        let operations = vec![
            EchoAst::IndexAccess {
                object: Box::new(EchoAst::Identifier("arr".to_string())),
                index: Box::new(EchoAst::Number(0)),
            },
            EchoAst::PropertyAccess {
                object: Box::new(EchoAst::Identifier("obj".to_string())),
                property: "field".to_string(),
            },
        ];
        
        for op in operations {
            match jit.compile_ast(&op) {
                Ok(()) => println!("Successfully compiled: {:?}", op),
                Err(e) => {
                    // Access operations require runtime object lookup
                    if e.to_string().contains("falling back to interpreter") {
                        println!("Access operation falls back to interpreter as expected: {:?}", op);
                    } else {
                        panic!("Failed to compile {:?}: {}", op, e);
                    }
                }
            }
        }
    }
}