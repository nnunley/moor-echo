//! Tests for JIT compilation of literal values

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
    fn test_jit_float() {
        let mut jit = create_test_jit();
        
        // Create a player
        let player_id = jit.create_player("test_player").unwrap();
        jit.switch_player(player_id).unwrap();
        
        // Test float literal
        let ast = EchoAst::Float(3.14159);
        let result = jit.eval(&ast).unwrap();
        
        match result {
            Value::Float(f) => assert_eq!(f, 3.14159),
            _ => panic!("Expected float value"),
        }
    }

    #[test]
    fn test_jit_string() {
        let mut jit = create_test_jit();
        
        // Create a player
        let player_id = jit.create_player("test_player").unwrap();
        jit.switch_player(player_id).unwrap();
        
        // Test string literal
        let ast = EchoAst::String("Hello, JIT!".to_string());
        let result = jit.eval(&ast).unwrap();
        
        match result {
            Value::String(s) => assert_eq!(s, "Hello, JIT!"),
            _ => panic!("Expected string value"),
        }
    }

    #[test]
    fn test_jit_boolean_true() {
        let mut jit = create_test_jit();
        
        // Create a player
        let player_id = jit.create_player("test_player").unwrap();
        jit.switch_player(player_id).unwrap();
        
        // Test boolean true
        let ast = EchoAst::Boolean(true);
        let result = jit.eval(&ast).unwrap();
        
        match result {
            Value::Boolean(b) => assert_eq!(b, true),
            _ => panic!("Expected boolean value"),
        }
    }

    #[test]
    fn test_jit_boolean_false() {
        let mut jit = create_test_jit();
        
        // Create a player
        let player_id = jit.create_player("test_player").unwrap();
        jit.switch_player(player_id).unwrap();
        
        // Test boolean false
        let ast = EchoAst::Boolean(false);
        let result = jit.eval(&ast).unwrap();
        
        match result {
            Value::Boolean(b) => assert_eq!(b, false),
            _ => panic!("Expected boolean value"),
        }
    }

    #[test]
    fn test_jit_null() {
        let mut jit = create_test_jit();
        
        // Create a player
        let player_id = jit.create_player("test_player").unwrap();
        jit.switch_player(player_id).unwrap();
        
        // Test null literal
        let ast = EchoAst::Null;
        let result = jit.eval(&ast).unwrap();
        
        match result {
            Value::Null => {},
            _ => panic!("Expected null value"),
        }
    }

    #[test]
    fn test_jit_compile_literals() {
        let mut jit = create_test_jit();
        
        // Skip test if JIT is not enabled
        if !jit.is_jit_enabled() {
            println!("Skipping literal compilation test - JIT not enabled");
            return;
        }
        
        // Test that compile_ast succeeds for all literal types
        let literals = vec![
            EchoAst::Float(2.71828),
            EchoAst::String("test".to_string()),
            EchoAst::Boolean(true),
            EchoAst::Boolean(false),
            EchoAst::Null,
        ];
        
        for lit in literals {
            match jit.compile_ast(&lit) {
                Ok(()) => println!("Successfully compiled: {:?}", lit),
                Err(e) => {
                    // These literals require type system support
                    // They fall back to interpreter by design
                    if e.to_string().contains("falling back to interpreter") {
                        println!("Expected fallback to interpreter: {:?}", lit);
                    } else {
                        panic!("Unexpected compilation error for {:?}: {}", lit, e);
                    }
                }
            }
        }
    }
    
    #[test]
    fn test_jit_arithmetic_with_floats() {
        let mut jit = create_test_jit();
        
        // Create a player
        let player_id = jit.create_player("test_player").unwrap();
        jit.switch_player(player_id).unwrap();
        
        // Test float arithmetic once we implement float support
        let ast = EchoAst::Add {
            left: Box::new(EchoAst::Float(1.5)),
            right: Box::new(EchoAst::Float(2.5)),
        };
        
        // This will fall back to interpreter for now
        let result = jit.eval(&ast).unwrap();
        
        match result {
            Value::Float(f) => assert_eq!(f, 4.0),
            _ => panic!("Expected float value"),
        }
    }
}