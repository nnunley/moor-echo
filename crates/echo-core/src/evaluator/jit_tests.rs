//! Tests for JIT compiler functionality

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
        
        // Use fallback constructor that gracefully handles unsupported architectures
        JitEvaluator::new_with_fallback(storage)
    }

    #[test]
    fn test_jit_basic_number() {
        let mut jit = create_test_jit();
        
        // Create a player
        let player_id = jit.create_player("test_player").unwrap();
        jit.switch_player(player_id).unwrap();
        
        // Test evaluating a simple number
        let ast = EchoAst::Number(42);
        let result = jit.eval(&ast).unwrap();
        
        match result {
            Value::Integer(n) => assert_eq!(n, 42),
            _ => panic!("Expected integer value"),
        }
    }

    #[test]
    fn test_jit_basic_string() {
        let mut jit = create_test_jit();
        
        // Create a player
        let player_id = jit.create_player("test_player").unwrap();
        jit.switch_player(player_id).unwrap();
        
        // Test evaluating a string
        let ast = EchoAst::String("hello".to_string());
        let result = jit.eval(&ast).unwrap();
        
        match result {
            Value::String(s) => assert_eq!(s, "hello"),
            _ => panic!("Expected string value"),
        }
    }

    #[test]
    fn test_jit_addition() {
        let mut jit = create_test_jit();
        
        // Create a player
        let player_id = jit.create_player("test_player").unwrap();
        jit.switch_player(player_id).unwrap();
        
        // Test addition
        let ast = EchoAst::Add {
            left: Box::new(EchoAst::Number(10)),
            right: Box::new(EchoAst::Number(32)),
        };
        let result = jit.eval(&ast).unwrap();
        
        match result {
            Value::Integer(n) => assert_eq!(n, 42),
            _ => panic!("Expected integer value"),
        }
    }

    #[test]
    fn test_jit_stats() {
        let jit = create_test_jit();
        
        let stats = jit.stats();
        assert_eq!(stats.compilation_count, 0);
        assert_eq!(stats.execution_count, 0);
        assert_eq!(stats.compiled_functions, 0);
        assert_eq!(stats.hot_threshold, 10);
        // JIT may be enabled or disabled depending on architecture and platform
        println!("JIT enabled: {} (architecture: {})", stats.jit_enabled, 
                 std::env::consts::ARCH);
        assert_eq!(stats.jit_enabled, jit.is_jit_enabled());
    }

    #[test]
    fn test_compile_number() {
        let mut jit = create_test_jit();
        
        // Test that compile_ast doesn't panic for a number
        let ast = EchoAst::Number(42);
        // Handle case where JIT is not enabled on this architecture
        match jit.compile_ast(&ast) {
            Ok(()) => println!("JIT compilation succeeded"),
            Err(e) if e.to_string().contains("not enabled") => {
                println!("JIT compilation disabled: {}", e);
            }
            Err(e) => panic!("Unexpected compilation error: {}", e),
        }
    }

    #[test]
    fn test_compile_addition() {
        let mut jit = create_test_jit();
        
        // Test that compile_ast doesn't panic for addition
        let ast = EchoAst::Add {
            left: Box::new(EchoAst::Number(10)),
            right: Box::new(EchoAst::Number(32)),
        };
        // Handle case where JIT is not enabled on this architecture
        match jit.compile_ast(&ast) {
            Ok(()) => println!("JIT compilation succeeded"),
            Err(e) if e.to_string().contains("not enabled") => {
                println!("JIT compilation disabled: {}", e);
            }
            Err(e) => panic!("Unexpected compilation error: {}", e),
        }
    }

    #[test]
    fn test_jit_compilation_count() {
        let mut jit = create_test_jit();
        
        // Skip test if JIT is not enabled
        if !jit.is_jit_enabled() {
            println!("Skipping compilation count test - JIT not enabled");
            return;
        }
        
        // Create a player
        let player_id = jit.create_player("test_player").unwrap();
        jit.switch_player(player_id).unwrap();
        
        // Initial stats should show 0 compilations
        let initial_stats = jit.stats();
        assert_eq!(initial_stats.compilation_count, 0);
        
        // Evaluate a number (should trigger compilation)
        let ast = EchoAst::Number(42);
        let _ = jit.eval(&ast).unwrap();
        
        // Check that compilation count increased
        let stats_after = jit.stats();
        assert_eq!(stats_after.compilation_count, 1, "Expected compilation count to increase");
        
        // Evaluate an addition (should trigger another compilation)
        let add_ast = EchoAst::Add {
            left: Box::new(EchoAst::Number(10)),
            right: Box::new(EchoAst::Number(32)),
        };
        let _ = jit.eval(&add_ast).unwrap();
        
        // Check that compilation count increased again
        // Note: Add compiles itself and its left/right operands, so we expect 4 total
        let final_stats = jit.stats();
        assert_eq!(final_stats.compilation_count, 4, "Expected compilation count to be 4 (1 number + 1 add + 2 operands)");
    }
}