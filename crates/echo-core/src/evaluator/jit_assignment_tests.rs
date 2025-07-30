//! Tests for JIT compilation of assignment operations

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
    fn test_jit_local_assignment() {
        let mut jit = create_test_jit();
        
        // Create a player
        let player_id = jit.create_player("test_player").unwrap();
        jit.switch_player(player_id).unwrap();
        
        // Test let x = 42
        let ast = EchoAst::LocalAssignment {
            target: BindingPattern::Identifier("x".to_string()),
            value: Box::new(EchoAst::Number(42)),
        };
        let result = jit.eval(&ast).unwrap();
        
        match result {
            Value::Integer(n) => assert_eq!(n, 42),
            _ => panic!("Expected integer value"),
        }
        
        // Verify the variable was assigned
        let check_ast = EchoAst::Identifier("x".to_string());
        let check_result = jit.eval(&check_ast).unwrap();
        match check_result {
            Value::Integer(n) => assert_eq!(n, 42),
            _ => panic!("Expected integer value"),
        }
    }

    #[test]
    fn test_jit_const_assignment() {
        let mut jit = create_test_jit();
        
        // Create a player
        let player_id = jit.create_player("test_player").unwrap();
        jit.switch_player(player_id).unwrap();
        
        // Test const PI = 3.14
        let ast = EchoAst::ConstAssignment {
            target: BindingPattern::Identifier("PI".to_string()),
            value: Box::new(EchoAst::Float(3.14)),
        };
        let result = jit.eval(&ast).unwrap();
        
        match result {
            Value::Float(f) => assert_eq!(f, 3.14),
            _ => panic!("Expected float value"),
        }
        
        // Verify the constant was assigned
        let check_ast = EchoAst::Identifier("PI".to_string());
        let check_result = jit.eval(&check_ast).unwrap();
        match check_result {
            Value::Float(f) => assert_eq!(f, 3.14),
            _ => panic!("Expected float value"),
        }
    }

    #[test]
    fn test_jit_local_assignment_pattern() {
        let mut jit = create_test_jit();
        
        // Create a player
        let player_id = jit.create_player("test_player").unwrap();
        jit.switch_player(player_id).unwrap();
        
        // Test let {x, y} = {x: 10, y: 20}
        let map_ast = EchoAst::Map {
            entries: vec![
                ("x".to_string(), EchoAst::Number(10)),
                ("y".to_string(), EchoAst::Number(20)),
            ],
        };
        
        let ast = EchoAst::LocalAssignment {
            target: BindingPattern::Object(vec![
                ("x".to_string(), BindingPattern::Identifier("x".to_string())),
                ("y".to_string(), BindingPattern::Identifier("y".to_string())),
            ]),
            value: Box::new(map_ast),
        };
        let _result = jit.eval(&ast).unwrap();
        
        // Verify the variables were assigned
        let check_x = EchoAst::Identifier("x".to_string());
        let result_x = jit.eval(&check_x).unwrap();
        match result_x {
            Value::Integer(n) => assert_eq!(n, 10),
            _ => panic!("Expected integer value for x"),
        }
        
        let check_y = EchoAst::Identifier("y".to_string());
        let result_y = jit.eval(&check_y).unwrap();
        match result_y {
            Value::Integer(n) => assert_eq!(n, 20),
            _ => panic!("Expected integer value for y"),
        }
    }

    #[test]
    fn test_jit_compile_assignments() {
        let mut jit = create_test_jit();
        
        // Skip test if JIT is not enabled
        if !jit.is_jit_enabled() {
            println!("Skipping assignment compilation test - JIT not enabled");
            return;
        }
        
        // Test that compile_ast handles assignment operations
        let operations = vec![
            EchoAst::LocalAssignment {
                target: BindingPattern::Identifier("x".to_string()),
                value: Box::new(EchoAst::Number(42)),
            },
            EchoAst::ConstAssignment {
                target: BindingPattern::Identifier("PI".to_string()),
                value: Box::new(EchoAst::Float(3.14)),
            },
        ];
        
        for op in operations {
            match jit.compile_ast(&op) {
                Ok(()) => println!("Successfully compiled: {:?}", op),
                Err(e) => {
                    // Assignment operations require runtime environment access
                    if e.to_string().contains("falling back to interpreter") {
                        println!("Assignment operation falls back to interpreter as expected: {:?}", op);
                    } else {
                        panic!("Failed to compile {:?}: {}", op, e);
                    }
                }
            }
        }
    }
}