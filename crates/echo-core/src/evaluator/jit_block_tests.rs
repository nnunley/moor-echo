//! Tests for JIT compilation of block statements

#[cfg(all(test, feature = "jit"))]
mod tests {
    use super::super::*;
    use crate::ast::{EchoAst, BindingPattern};
    use crate::storage::Storage;
    use std::sync::Arc;
    use tempfile::TempDir;

    fn create_test_jit() -> JitEvaluator {
        let temp_dir = TempDir::new().unwrap();
        let storage = Arc::new(Storage::new(temp_dir.path()).unwrap());
        JitEvaluator::new_with_fallback(storage)
    }

    #[test]
    fn test_jit_block_empty() {
        let mut jit = create_test_jit();
        
        // Create a player
        let player_id = jit.create_player("test_player").unwrap();
        jit.switch_player(player_id).unwrap();
        
        // Test empty block
        let ast = EchoAst::Block(vec![]);
        let result = jit.eval(&ast).unwrap();
        
        match result {
            Value::Null => {}, // Empty block returns null
            _ => panic!("Expected null value"),
        }
    }

    #[test]
    fn test_jit_block_single_expression() {
        let mut jit = create_test_jit();
        
        // Create a player
        let player_id = jit.create_player("test_player").unwrap();
        jit.switch_player(player_id).unwrap();
        
        // Test block with single expression
        let ast = EchoAst::Block(vec![
            EchoAst::Number(42),
        ]);
        let result = jit.eval(&ast).unwrap();
        
        match result {
            Value::Integer(n) => assert_eq!(n, 42),
            _ => panic!("Expected integer value"),
        }
    }

    #[test]
    fn test_jit_block_multiple_statements() {
        let mut jit = create_test_jit();
        
        // Create a player
        let player_id = jit.create_player("test_player").unwrap();
        jit.switch_player(player_id).unwrap();
        
        // Test block with multiple statements
        let ast = EchoAst::Block(vec![
            EchoAst::LocalAssignment {
                target: BindingPattern::Identifier("x".to_string()),
                value: Box::new(EchoAst::Number(10)),
            },
            EchoAst::LocalAssignment {
                target: BindingPattern::Identifier("y".to_string()),
                value: Box::new(EchoAst::Number(20)),
            },
            EchoAst::Add {
                left: Box::new(EchoAst::Identifier("x".to_string())),
                right: Box::new(EchoAst::Identifier("y".to_string())),
            },
        ]);
        let result = jit.eval(&ast).unwrap();
        
        match result {
            Value::Integer(n) => assert_eq!(n, 30),
            _ => panic!("Expected integer value"),
        }
    }

    #[test]
    fn test_jit_block_scoping() {
        let mut jit = create_test_jit();
        
        // Create a player
        let player_id = jit.create_player("test_player").unwrap();
        jit.switch_player(player_id).unwrap();
        
        // First set a variable outside the block
        let setup_ast = EchoAst::LocalAssignment {
            target: BindingPattern::Identifier("x".to_string()),
            value: Box::new(EchoAst::Number(10)),
        };
        jit.eval(&setup_ast).unwrap();
        
        // Test block that shadows the variable
        let block_ast = EchoAst::Block(vec![
            EchoAst::LocalAssignment {
                target: BindingPattern::Identifier("x".to_string()),
                value: Box::new(EchoAst::Number(20)),
            },
            EchoAst::Identifier("x".to_string()),
        ]);
        let result = jit.eval(&block_ast).unwrap();
        
        match result {
            Value::Integer(n) => assert_eq!(n, 20), // Should be the shadowed value
            _ => panic!("Expected integer value"),
        }
        
        // Check that the outer variable is still 10
        // (Note: This depends on whether blocks create new scopes in Echo)
        let check_ast = EchoAst::Identifier("x".to_string());
        let outer_result = jit.eval(&check_ast).unwrap();
        match outer_result {
            Value::Integer(n) => {
                // If blocks don't create new scopes, this will be 20
                // If they do create new scopes, this will be 10
                println!("Outer x = {}", n);
            }
            _ => panic!("Expected integer value"),
        }
    }

    #[test]
    fn test_jit_compile_blocks() {
        let mut jit = create_test_jit();
        
        // Skip test if JIT is not enabled
        if !jit.is_jit_enabled() {
            println!("Skipping block compilation test - JIT not enabled");
            return;
        }
        
        // Test that compile_ast handles block operations
        let operations = vec![
            EchoAst::Block(vec![]),
            EchoAst::Block(vec![
                EchoAst::Number(42),
            ]),
            EchoAst::Block(vec![
                EchoAst::Number(1),
                EchoAst::Number(2),
                EchoAst::Number(3),
            ]),
        ];
        
        for op in operations {
            match jit.compile_ast(&op) {
                Ok(()) => println!("Successfully compiled: {:?}", op),
                Err(e) => {
                    // Block statements may fall back to interpreter
                    if e.to_string().contains("falling back to interpreter") {
                        println!("Block operation falls back to interpreter as expected: {:?}", op);
                    } else {
                        panic!("Failed to compile {:?}: {}", op, e);
                    }
                }
            }
        }
    }
}