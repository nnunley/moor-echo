//! Tests for JIT compilation of arithmetic operations

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
    fn test_jit_subtract() {
        let mut jit = create_test_jit();
        
        // Create a player
        let player_id = jit.create_player("test_player").unwrap();
        jit.switch_player(player_id).unwrap();
        
        // Test subtraction
        let ast = EchoAst::Subtract {
            left: Box::new(EchoAst::Number(42)),
            right: Box::new(EchoAst::Number(10)),
        };
        let result = jit.eval(&ast).unwrap();
        
        match result {
            Value::Integer(n) => assert_eq!(n, 32),
            _ => panic!("Expected integer value"),
        }
    }

    #[test]
    fn test_jit_multiply() {
        let mut jit = create_test_jit();
        
        // Create a player
        let player_id = jit.create_player("test_player").unwrap();
        jit.switch_player(player_id).unwrap();
        
        // Test multiplication
        let ast = EchoAst::Multiply {
            left: Box::new(EchoAst::Number(6)),
            right: Box::new(EchoAst::Number(7)),
        };
        let result = jit.eval(&ast).unwrap();
        
        match result {
            Value::Integer(n) => assert_eq!(n, 42),
            _ => panic!("Expected integer value"),
        }
    }

    #[test]
    fn test_jit_divide() {
        let mut jit = create_test_jit();
        
        // Create a player
        let player_id = jit.create_player("test_player").unwrap();
        jit.switch_player(player_id).unwrap();
        
        // Test division
        let ast = EchoAst::Divide {
            left: Box::new(EchoAst::Number(84)),
            right: Box::new(EchoAst::Number(2)),
        };
        let result = jit.eval(&ast).unwrap();
        
        match result {
            Value::Integer(n) => assert_eq!(n, 42),
            _ => panic!("Expected integer value"),
        }
    }

    #[test]
    fn test_jit_modulo() {
        let mut jit = create_test_jit();
        
        // Create a player
        let player_id = jit.create_player("test_player").unwrap();
        jit.switch_player(player_id).unwrap();
        
        // Test modulo
        let ast = EchoAst::Modulo {
            left: Box::new(EchoAst::Number(17)),
            right: Box::new(EchoAst::Number(5)),
        };
        let result = jit.eval(&ast).unwrap();
        
        match result {
            Value::Integer(n) => assert_eq!(n, 2),
            _ => panic!("Expected integer value"),
        }
    }

    #[test]
    fn test_jit_power() {
        let mut jit = create_test_jit();
        
        // Create a player
        let player_id = jit.create_player("test_player").unwrap();
        jit.switch_player(player_id).unwrap();
        
        // Test power (2^5 = 32)
        let ast = EchoAst::Power {
            left: Box::new(EchoAst::Number(2)),
            right: Box::new(EchoAst::Number(5)),
        };
        let result = jit.eval(&ast).unwrap();
        
        match result {
            Value::Integer(n) => assert_eq!(n, 32),
            _ => panic!("Expected integer value"),
        }
    }

    #[test]
    fn test_jit_unary_minus() {
        let mut jit = create_test_jit();
        
        // Create a player
        let player_id = jit.create_player("test_player").unwrap();
        jit.switch_player(player_id).unwrap();
        
        // Test unary minus
        let ast = EchoAst::UnaryMinus {
            operand: Box::new(EchoAst::Number(42)),
        };
        let result = jit.eval(&ast).unwrap();
        
        match result {
            Value::Integer(n) => assert_eq!(n, -42),
            _ => panic!("Expected integer value"),
        }
    }

    #[test]
    fn test_jit_unary_plus() {
        let mut jit = create_test_jit();
        
        // Create a player
        let player_id = jit.create_player("test_player").unwrap();
        jit.switch_player(player_id).unwrap();
        
        // Test unary plus
        let ast = EchoAst::UnaryPlus {
            operand: Box::new(EchoAst::Number(42)),
        };
        let result = jit.eval(&ast).unwrap();
        
        match result {
            Value::Integer(n) => assert_eq!(n, 42),
            _ => panic!("Expected integer value"),
        }
    }

    #[test]
    fn test_jit_compile_arithmetic() {
        let mut jit = create_test_jit();
        
        // Skip test if JIT is not enabled
        if !jit.is_jit_enabled() {
            println!("Skipping arithmetic compilation test - JIT not enabled");
            return;
        }
        
        // Test that compile_ast succeeds for all arithmetic operations
        let operations = vec![
            EchoAst::Subtract {
                left: Box::new(EchoAst::Number(10)),
                right: Box::new(EchoAst::Number(5)),
            },
            EchoAst::Multiply {
                left: Box::new(EchoAst::Number(3)),
                right: Box::new(EchoAst::Number(4)),
            },
            EchoAst::Divide {
                left: Box::new(EchoAst::Number(20)),
                right: Box::new(EchoAst::Number(4)),
            },
            EchoAst::Modulo {
                left: Box::new(EchoAst::Number(13)),
                right: Box::new(EchoAst::Number(5)),
            },
            EchoAst::Power {
                left: Box::new(EchoAst::Number(2)),
                right: Box::new(EchoAst::Number(3)),
            },
            EchoAst::UnaryMinus {
                operand: Box::new(EchoAst::Number(42)),
            },
            EchoAst::UnaryPlus {
                operand: Box::new(EchoAst::Number(42)),
            },
        ];
        
        for op in operations {
            match jit.compile_ast(&op) {
                Ok(()) => println!("Successfully compiled: {:?}", op),
                Err(e) => {
                    // Power operation is expected to fall back to interpreter
                    if matches!(op, EchoAst::Power { .. }) && e.to_string().contains("falling back to interpreter") {
                        println!("Power operation falls back to interpreter as expected");
                    } else {
                        panic!("Failed to compile {:?}: {}", op, e);
                    }
                }
            }
        }
    }
}