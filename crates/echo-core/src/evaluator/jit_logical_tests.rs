//! Tests for JIT compilation of logical operations

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
    fn test_jit_and() {
        let mut jit = create_test_jit();
        
        // Create a player
        let player_id = jit.create_player("test_player").unwrap();
        jit.switch_player(player_id).unwrap();
        
        // Test AND - true && true = true
        let ast = EchoAst::And {
            left: Box::new(EchoAst::Boolean(true)),
            right: Box::new(EchoAst::Boolean(true)),
        };
        let result = jit.eval(&ast).unwrap();
        
        match result {
            Value::Boolean(b) => assert_eq!(b, true),
            _ => panic!("Expected boolean value"),
        }
        
        // Test AND - true && false = false
        let ast = EchoAst::And {
            left: Box::new(EchoAst::Boolean(true)),
            right: Box::new(EchoAst::Boolean(false)),
        };
        let result = jit.eval(&ast).unwrap();
        
        match result {
            Value::Boolean(b) => assert_eq!(b, false),
            _ => panic!("Expected boolean value"),
        }
        
        // Test AND - false && true = false (short-circuit)
        let ast = EchoAst::And {
            left: Box::new(EchoAst::Boolean(false)),
            right: Box::new(EchoAst::Boolean(true)),
        };
        let result = jit.eval(&ast).unwrap();
        
        match result {
            Value::Boolean(b) => assert_eq!(b, false),
            _ => panic!("Expected boolean value"),
        }
    }

    #[test]
    fn test_jit_or() {
        let mut jit = create_test_jit();
        
        // Create a player
        let player_id = jit.create_player("test_player").unwrap();
        jit.switch_player(player_id).unwrap();
        
        // Test OR - true || false = true
        let ast = EchoAst::Or {
            left: Box::new(EchoAst::Boolean(true)),
            right: Box::new(EchoAst::Boolean(false)),
        };
        let result = jit.eval(&ast).unwrap();
        
        match result {
            Value::Boolean(b) => assert_eq!(b, true),
            _ => panic!("Expected boolean value"),
        }
        
        // Test OR - false || true = true
        let ast = EchoAst::Or {
            left: Box::new(EchoAst::Boolean(false)),
            right: Box::new(EchoAst::Boolean(true)),
        };
        let result = jit.eval(&ast).unwrap();
        
        match result {
            Value::Boolean(b) => assert_eq!(b, true),
            _ => panic!("Expected boolean value"),
        }
        
        // Test OR - false || false = false
        let ast = EchoAst::Or {
            left: Box::new(EchoAst::Boolean(false)),
            right: Box::new(EchoAst::Boolean(false)),
        };
        let result = jit.eval(&ast).unwrap();
        
        match result {
            Value::Boolean(b) => assert_eq!(b, false),
            _ => panic!("Expected boolean value"),
        }
        
        // Test OR - true || true = true (short-circuit)
        let ast = EchoAst::Or {
            left: Box::new(EchoAst::Boolean(true)),
            right: Box::new(EchoAst::Boolean(true)),
        };
        let result = jit.eval(&ast).unwrap();
        
        match result {
            Value::Boolean(b) => assert_eq!(b, true),
            _ => panic!("Expected boolean value"),
        }
    }

    #[test]
    fn test_jit_not() {
        let mut jit = create_test_jit();
        
        // Create a player
        let player_id = jit.create_player("test_player").unwrap();
        jit.switch_player(player_id).unwrap();
        
        // Test NOT - !true = false
        let ast = EchoAst::Not {
            operand: Box::new(EchoAst::Boolean(true)),
        };
        let result = jit.eval(&ast).unwrap();
        
        match result {
            Value::Boolean(b) => assert_eq!(b, false),
            _ => panic!("Expected boolean value"),
        }
        
        // Test NOT - !false = true
        let ast = EchoAst::Not {
            operand: Box::new(EchoAst::Boolean(false)),
        };
        let result = jit.eval(&ast).unwrap();
        
        match result {
            Value::Boolean(b) => assert_eq!(b, true),
            _ => panic!("Expected boolean value"),
        }
    }

    #[test]
    fn test_jit_logical_with_comparisons() {
        let mut jit = create_test_jit();
        
        // Create a player
        let player_id = jit.create_player("test_player").unwrap();
        jit.switch_player(player_id).unwrap();
        
        // Test (10 < 20) && (5 > 2) = true && true = true
        let ast = EchoAst::And {
            left: Box::new(EchoAst::LessThan {
                left: Box::new(EchoAst::Number(10)),
                right: Box::new(EchoAst::Number(20)),
            }),
            right: Box::new(EchoAst::GreaterThan {
                left: Box::new(EchoAst::Number(5)),
                right: Box::new(EchoAst::Number(2)),
            }),
        };
        let result = jit.eval(&ast).unwrap();
        
        match result {
            Value::Boolean(b) => assert_eq!(b, true),
            _ => panic!("Expected boolean value"),
        }
        
        // Test (10 > 20) || (5 == 5) = false || true = true
        let ast = EchoAst::Or {
            left: Box::new(EchoAst::GreaterThan {
                left: Box::new(EchoAst::Number(10)),
                right: Box::new(EchoAst::Number(20)),
            }),
            right: Box::new(EchoAst::Equal {
                left: Box::new(EchoAst::Number(5)),
                right: Box::new(EchoAst::Number(5)),
            }),
        };
        let result = jit.eval(&ast).unwrap();
        
        match result {
            Value::Boolean(b) => assert_eq!(b, true),
            _ => panic!("Expected boolean value"),
        }
    }

    #[test]
    fn test_jit_compile_logical() {
        let mut jit = create_test_jit();
        
        // Skip test if JIT is not enabled
        if !jit.is_jit_enabled() {
            println!("Skipping logical compilation test - JIT not enabled");
            return;
        }
        
        // Test that compile_ast succeeds for all logical operations
        let operations = vec![
            EchoAst::And {
                left: Box::new(EchoAst::Boolean(true)),
                right: Box::new(EchoAst::Boolean(false)),
            },
            EchoAst::Or {
                left: Box::new(EchoAst::Boolean(false)),
                right: Box::new(EchoAst::Boolean(true)),
            },
            EchoAst::Not {
                operand: Box::new(EchoAst::Boolean(true)),
            },
            // Test with nested comparisons
            EchoAst::And {
                left: Box::new(EchoAst::LessThan {
                    left: Box::new(EchoAst::Number(1)),
                    right: Box::new(EchoAst::Number(2)),
                }),
                right: Box::new(EchoAst::GreaterThan {
                    left: Box::new(EchoAst::Number(3)),
                    right: Box::new(EchoAst::Number(2)),
                }),
            },
        ];
        
        for op in operations {
            match jit.compile_ast(&op) {
                Ok(()) => println!("Successfully compiled: {:?}", op),
                Err(e) => {
                    // Logical operations mostly fall back to interpreter
                    if e.to_string().contains("falling back to interpreter") {
                        println!("Logical operation falls back to interpreter as expected: {:?}", op);
                    } else {
                        panic!("Failed to compile {:?}: {}", op, e);
                    }
                }
            }
        }
    }
    
    #[test]
    fn test_jit_short_circuit_and() {
        let mut jit = create_test_jit();
        
        // Create a player
        let player_id = jit.create_player("test_player").unwrap();
        jit.switch_player(player_id).unwrap();
        
        // Test short-circuit AND - false && (would error) = false
        // If the right side were evaluated, it would error (division by zero)
        let ast = EchoAst::And {
            left: Box::new(EchoAst::Boolean(false)),
            right: Box::new(EchoAst::Divide {
                left: Box::new(EchoAst::Number(1)),
                right: Box::new(EchoAst::Number(0)),
            }),
        };
        let result = jit.eval(&ast).unwrap();
        
        match result {
            Value::Boolean(b) => assert_eq!(b, false),
            _ => panic!("Expected boolean value"),
        }
    }
    
    #[test]
    fn test_jit_short_circuit_or() {
        let mut jit = create_test_jit();
        
        // Create a player
        let player_id = jit.create_player("test_player").unwrap();
        jit.switch_player(player_id).unwrap();
        
        // Test short-circuit OR - true || (would error) = true
        // If the right side were evaluated, it would error (division by zero)
        let ast = EchoAst::Or {
            left: Box::new(EchoAst::Boolean(true)),
            right: Box::new(EchoAst::Divide {
                left: Box::new(EchoAst::Number(1)),
                right: Box::new(EchoAst::Number(0)),
            }),
        };
        let result = jit.eval(&ast).unwrap();
        
        match result {
            Value::Boolean(b) => assert_eq!(b, true),
            _ => panic!("Expected boolean value"),
        }
    }
}