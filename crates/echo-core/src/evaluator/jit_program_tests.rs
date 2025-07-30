//! Tests for JIT compilation of ExpressionStatement and Program

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
    fn test_jit_expression_statement() {
        let mut jit = create_test_jit();
        
        // Create a player
        let player_id = jit.create_player("test_player").unwrap();
        jit.switch_player(player_id).unwrap();
        
        // Test expression statement
        let ast = EchoAst::ExpressionStatement(Box::new(EchoAst::Number(42)));
        let result = jit.eval(&ast).unwrap();
        
        match result {
            Value::Integer(n) => assert_eq!(n, 42),
            _ => panic!("Expected integer value"),
        }
    }

    #[test]
    fn test_jit_program_empty() {
        let mut jit = create_test_jit();
        
        // Create a player
        let player_id = jit.create_player("test_player").unwrap();
        jit.switch_player(player_id).unwrap();
        
        // Test empty program
        let ast = EchoAst::Program(vec![]);
        let result = jit.eval(&ast).unwrap();
        
        match result {
            Value::Null => {}, // Empty program returns null
            _ => panic!("Expected null value"),
        }
    }

    #[test]
    fn test_jit_program_single_statement() {
        let mut jit = create_test_jit();
        
        // Create a player
        let player_id = jit.create_player("test_player").unwrap();
        jit.switch_player(player_id).unwrap();
        
        // Test program with single statement
        let ast = EchoAst::Program(vec![
            EchoAst::Number(42),
        ]);
        let result = jit.eval(&ast).unwrap();
        
        match result {
            Value::Integer(n) => assert_eq!(n, 42),
            _ => panic!("Expected integer value"),
        }
    }

    #[test]
    fn test_jit_program_multiple_statements() {
        let mut jit = create_test_jit();
        
        // Create a player
        let player_id = jit.create_player("test_player").unwrap();
        jit.switch_player(player_id).unwrap();
        
        // Test program with multiple statements
        let ast = EchoAst::Program(vec![
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
    fn test_jit_compile_expression_statement() {
        let mut jit = create_test_jit();
        
        // Skip test if JIT is not enabled
        if !jit.is_jit_enabled() {
            println!("Skipping expression statement compilation test - JIT not enabled");
            return;
        }
        
        // Test that compile_ast handles expression statement
        let ast = EchoAst::ExpressionStatement(Box::new(EchoAst::Number(42)));
        match jit.compile_ast(&ast) {
            Ok(()) => println!("Successfully compiled: {:?}", ast),
            Err(e) => {
                if e.to_string().contains("falling back to interpreter") {
                    println!("ExpressionStatement falls back to interpreter as expected: {:?}", ast);
                } else {
                    panic!("Failed to compile {:?}: {}", ast, e);
                }
            }
        }
    }

    #[test]
    fn test_jit_compile_program() {
        let mut jit = create_test_jit();
        
        // Skip test if JIT is not enabled
        if !jit.is_jit_enabled() {
            println!("Skipping program compilation test - JIT not enabled");
            return;
        }
        
        // Test that compile_ast handles program operations
        let operations = vec![
            EchoAst::Program(vec![]),
            EchoAst::Program(vec![
                EchoAst::Number(42),
            ]),
            EchoAst::Program(vec![
                EchoAst::Number(1),
                EchoAst::Number(2),
                EchoAst::Number(3),
            ]),
        ];
        
        for op in operations {
            match jit.compile_ast(&op) {
                Ok(()) => println!("Successfully compiled: {:?}", op),
                Err(e) => {
                    if e.to_string().contains("falling back to interpreter") {
                        println!("Program falls back to interpreter as expected: {:?}", op);
                    } else {
                        panic!("Failed to compile {:?}: {}", op, e);
                    }
                }
            }
        }
    }
}