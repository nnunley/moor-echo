//! Tests for JIT compilation of comparison operations

#[cfg(all(test, feature = "jit"))]
mod tests {
    use super::super::*;
    use crate::ast::EchoAst;
    use crate::storage::Storage;
    use std::sync::Arc;
    use tempfile::TempDir;

    fn create_test_jit() -> JitEvaluator {
        let temp_dir = TempDir::new().unwrap();
        let storage = Arc::new(Storage::new(temp_dir.path()).unwrap());
        JitEvaluator::new_with_fallback(storage)
    }

    #[test]
    fn test_jit_equal() {
        let mut jit = create_test_jit();
        
        // Create a player
        let player_id = jit.create_player("test_player").unwrap();
        jit.switch_player(player_id).unwrap();
        
        // Test equality - true case
        let ast = EchoAst::Equal {
            left: Box::new(EchoAst::Number(42)),
            right: Box::new(EchoAst::Number(42)),
        };
        let result = jit.eval(&ast).unwrap();
        
        match result {
            Value::Boolean(b) => assert_eq!(b, true),
            _ => panic!("Expected boolean value"),
        }
        
        // Test equality - false case
        let ast = EchoAst::Equal {
            left: Box::new(EchoAst::Number(42)),
            right: Box::new(EchoAst::Number(24)),
        };
        let result = jit.eval(&ast).unwrap();
        
        match result {
            Value::Boolean(b) => assert_eq!(b, false),
            _ => panic!("Expected boolean value"),
        }
    }

    #[test]
    fn test_jit_not_equal() {
        let mut jit = create_test_jit();
        
        // Create a player
        let player_id = jit.create_player("test_player").unwrap();
        jit.switch_player(player_id).unwrap();
        
        // Test inequality - true case
        let ast = EchoAst::NotEqual {
            left: Box::new(EchoAst::Number(42)),
            right: Box::new(EchoAst::Number(24)),
        };
        let result = jit.eval(&ast).unwrap();
        
        match result {
            Value::Boolean(b) => assert_eq!(b, true),
            _ => panic!("Expected boolean value"),
        }
        
        // Test inequality - false case
        let ast = EchoAst::NotEqual {
            left: Box::new(EchoAst::Number(42)),
            right: Box::new(EchoAst::Number(42)),
        };
        let result = jit.eval(&ast).unwrap();
        
        match result {
            Value::Boolean(b) => assert_eq!(b, false),
            _ => panic!("Expected boolean value"),
        }
    }

    #[test]
    fn test_jit_less_than() {
        let mut jit = create_test_jit();
        
        // Create a player
        let player_id = jit.create_player("test_player").unwrap();
        jit.switch_player(player_id).unwrap();
        
        // Test less than - true case
        let ast = EchoAst::LessThan {
            left: Box::new(EchoAst::Number(10)),
            right: Box::new(EchoAst::Number(20)),
        };
        let result = jit.eval(&ast).unwrap();
        
        match result {
            Value::Boolean(b) => assert_eq!(b, true),
            _ => panic!("Expected boolean value"),
        }
        
        // Test less than - false case
        let ast = EchoAst::LessThan {
            left: Box::new(EchoAst::Number(20)),
            right: Box::new(EchoAst::Number(10)),
        };
        let result = jit.eval(&ast).unwrap();
        
        match result {
            Value::Boolean(b) => assert_eq!(b, false),
            _ => panic!("Expected boolean value"),
        }
    }

    #[test]
    fn test_jit_less_equal() {
        let mut jit = create_test_jit();
        
        // Create a player
        let player_id = jit.create_player("test_player").unwrap();
        jit.switch_player(player_id).unwrap();
        
        // Test less than or equal - true cases
        let ast = EchoAst::LessEqual {
            left: Box::new(EchoAst::Number(10)),
            right: Box::new(EchoAst::Number(20)),
        };
        let result = jit.eval(&ast).unwrap();
        
        match result {
            Value::Boolean(b) => assert_eq!(b, true),
            _ => panic!("Expected boolean value"),
        }
        
        let ast = EchoAst::LessEqual {
            left: Box::new(EchoAst::Number(20)),
            right: Box::new(EchoAst::Number(20)),
        };
        let result = jit.eval(&ast).unwrap();
        
        match result {
            Value::Boolean(b) => assert_eq!(b, true),
            _ => panic!("Expected boolean value"),
        }
    }

    #[test]
    fn test_jit_greater_than() {
        let mut jit = create_test_jit();
        
        // Create a player
        let player_id = jit.create_player("test_player").unwrap();
        jit.switch_player(player_id).unwrap();
        
        // Test greater than - true case
        let ast = EchoAst::GreaterThan {
            left: Box::new(EchoAst::Number(20)),
            right: Box::new(EchoAst::Number(10)),
        };
        let result = jit.eval(&ast).unwrap();
        
        match result {
            Value::Boolean(b) => assert_eq!(b, true),
            _ => panic!("Expected boolean value"),
        }
        
        // Test greater than - false case
        let ast = EchoAst::GreaterThan {
            left: Box::new(EchoAst::Number(10)),
            right: Box::new(EchoAst::Number(20)),
        };
        let result = jit.eval(&ast).unwrap();
        
        match result {
            Value::Boolean(b) => assert_eq!(b, false),
            _ => panic!("Expected boolean value"),
        }
    }

    #[test]
    fn test_jit_greater_equal() {
        let mut jit = create_test_jit();
        
        // Create a player
        let player_id = jit.create_player("test_player").unwrap();
        jit.switch_player(player_id).unwrap();
        
        // Test greater than or equal - true cases
        let ast = EchoAst::GreaterEqual {
            left: Box::new(EchoAst::Number(20)),
            right: Box::new(EchoAst::Number(10)),
        };
        let result = jit.eval(&ast).unwrap();
        
        match result {
            Value::Boolean(b) => assert_eq!(b, true),
            _ => panic!("Expected boolean value"),
        }
        
        let ast = EchoAst::GreaterEqual {
            left: Box::new(EchoAst::Number(20)),
            right: Box::new(EchoAst::Number(20)),
        };
        let result = jit.eval(&ast).unwrap();
        
        match result {
            Value::Boolean(b) => assert_eq!(b, true),
            _ => panic!("Expected boolean value"),
        }
    }

    #[test]
    fn test_jit_in() {
        let mut jit = create_test_jit();
        
        // Create a player
        let player_id = jit.create_player("test_player").unwrap();
        jit.switch_player(player_id).unwrap();
        
        // Test 'in' operator with list
        let ast = EchoAst::In {
            left: Box::new(EchoAst::Number(2)),
            right: Box::new(EchoAst::List {
                elements: vec![
                    EchoAst::Number(1),
                    EchoAst::Number(2),
                    EchoAst::Number(3),
                ],
            }),
        };
        // This will fall back to interpreter since we don't support lists yet
        let result = jit.eval(&ast).unwrap();
        
        match result {
            Value::Boolean(b) => assert_eq!(b, true),
            _ => panic!("Expected boolean value"),
        }
    }

    #[test]
    fn test_jit_compile_comparisons() {
        let mut jit = create_test_jit();
        
        // Skip test if JIT is not enabled
        if !jit.is_jit_enabled() {
            println!("Skipping comparison compilation test - JIT not enabled");
            return;
        }
        
        // Test that compile_ast succeeds for all comparison operations
        let operations = vec![
            EchoAst::Equal {
                left: Box::new(EchoAst::Number(10)),
                right: Box::new(EchoAst::Number(10)),
            },
            EchoAst::NotEqual {
                left: Box::new(EchoAst::Number(10)),
                right: Box::new(EchoAst::Number(5)),
            },
            EchoAst::LessThan {
                left: Box::new(EchoAst::Number(5)),
                right: Box::new(EchoAst::Number(10)),
            },
            EchoAst::LessEqual {
                left: Box::new(EchoAst::Number(5)),
                right: Box::new(EchoAst::Number(10)),
            },
            EchoAst::GreaterThan {
                left: Box::new(EchoAst::Number(10)),
                right: Box::new(EchoAst::Number(5)),
            },
            EchoAst::GreaterEqual {
                left: Box::new(EchoAst::Number(10)),
                right: Box::new(EchoAst::Number(5)),
            },
            EchoAst::In {
                left: Box::new(EchoAst::Number(2)),
                right: Box::new(EchoAst::List {
                    elements: vec![EchoAst::Number(1), EchoAst::Number(2)],
                }),
            },
        ];
        
        for op in operations {
            match jit.compile_ast(&op) {
                Ok(()) => println!("Successfully compiled: {:?}", op),
                Err(e) => {
                    // In operator requires list support
                    if matches!(op, EchoAst::In { .. }) && e.to_string().contains("falling back to interpreter") {
                        println!("In operator falls back to interpreter as expected");
                    } else {
                        panic!("Failed to compile {:?}: {}", op, e);
                    }
                }
            }
        }
    }
    
    #[test]
    fn test_jit_comparison_with_floats() {
        let mut jit = create_test_jit();
        
        // Create a player
        let player_id = jit.create_player("test_player").unwrap();
        jit.switch_player(player_id).unwrap();
        
        // Test float comparison
        let ast = EchoAst::LessThan {
            left: Box::new(EchoAst::Float(1.5)),
            right: Box::new(EchoAst::Float(2.5)),
        };
        
        // This will fall back to interpreter for now
        let result = jit.eval(&ast).unwrap();
        
        match result {
            Value::Boolean(b) => assert_eq!(b, true),
            _ => panic!("Expected boolean value"),
        }
    }
}