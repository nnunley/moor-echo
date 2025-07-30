//! Tests for JIT compilation of variable operations

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
    fn test_jit_identifier_read() {
        let mut jit = create_test_jit();
        
        // Create a player
        let player_id = jit.create_player("test_player").unwrap();
        jit.switch_player(player_id).unwrap();
        
        // First, assign a value using the interpreter
        let assign_ast = EchoAst::Assignment {
            target: LValue::Binding {
                binding_type: BindingType::Let,
                pattern: BindingPattern::Identifier("x".to_string()),
            },
            value: Box::new(EchoAst::Number(42)),
        };
        jit.eval(&assign_ast).unwrap();
        
        // Now read the variable
        let ast = EchoAst::Identifier("x".to_string());
        let result = jit.eval(&ast).unwrap();
        
        match result {
            Value::Integer(n) => assert_eq!(n, 42),
            _ => panic!("Expected integer value"),
        }
    }

    #[test]
    fn test_jit_assignment() {
        let mut jit = create_test_jit();
        
        // Create a player
        let player_id = jit.create_player("test_player").unwrap();
        jit.switch_player(player_id).unwrap();
        
        // Test assignment
        let ast = EchoAst::Assignment {
            target: LValue::Binding {
                binding_type: BindingType::Let,
                pattern: BindingPattern::Identifier("y".to_string()),
            },
            value: Box::new(EchoAst::Number(100)),
        };
        let result = jit.eval(&ast).unwrap();
        
        // Assignment returns the assigned value
        match result {
            Value::Integer(n) => assert_eq!(n, 100),
            _ => panic!("Expected integer value"),
        }
        
        // Verify the variable was set
        let read_ast = EchoAst::Identifier("y".to_string());
        let read_result = jit.eval(&read_ast).unwrap();
        
        match read_result {
            Value::Integer(n) => assert_eq!(n, 100),
            _ => panic!("Expected integer value"),
        }
    }

    #[test]
    fn test_jit_compile_variables() {
        let mut jit = create_test_jit();
        
        // Skip test if JIT is not enabled
        if !jit.is_jit_enabled() {
            println!("Skipping variable compilation test - JIT not enabled");
            return;
        }
        
        // Test that compile_ast handles variable operations
        let operations = vec![
            EchoAst::Identifier("x".to_string()),
            EchoAst::Assignment {
                target: LValue::Binding {
                    binding_type: BindingType::Let,
                    pattern: BindingPattern::Identifier("z".to_string()),
                },
                value: Box::new(EchoAst::Number(50)),
            },
        ];
        
        for op in operations {
            match jit.compile_ast(&op) {
                Ok(()) => println!("Successfully compiled: {:?}", op),
                Err(e) => {
                    // Variable operations require runtime support
                    if e.to_string().contains("falling back to interpreter") {
                        println!("Variable operation falls back to interpreter as expected: {:?}", op);
                    } else {
                        panic!("Failed to compile {:?}: {}", op, e);
                    }
                }
            }
        }
    }
    
    #[test]
    fn test_jit_variable_in_expression() {
        let mut jit = create_test_jit();
        
        // Create a player
        let player_id = jit.create_player("test_player").unwrap();
        jit.switch_player(player_id).unwrap();
        
        // Set up variables
        let assign1 = EchoAst::Assignment {
            target: LValue::Binding {
                binding_type: BindingType::Let,
                pattern: BindingPattern::Identifier("a".to_string()),
            },
            value: Box::new(EchoAst::Number(10)),
        };
        jit.eval(&assign1).unwrap();
        
        let assign2 = EchoAst::Assignment {
            target: LValue::Binding {
                binding_type: BindingType::Let,
                pattern: BindingPattern::Identifier("b".to_string()),
            },
            value: Box::new(EchoAst::Number(20)),
        };
        jit.eval(&assign2).unwrap();
        
        // Test variable in arithmetic expression: a + b
        let ast = EchoAst::Add {
            left: Box::new(EchoAst::Identifier("a".to_string())),
            right: Box::new(EchoAst::Identifier("b".to_string())),
        };
        let result = jit.eval(&ast).unwrap();
        
        match result {
            Value::Integer(n) => assert_eq!(n, 30),
            _ => panic!("Expected integer value"),
        }
    }
}