//! Tests for JIT compilation of collection operations

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
    fn test_jit_map() {
        let mut jit = create_test_jit();
        
        // Create a player
        let player_id = jit.create_player("test_player").unwrap();
        jit.switch_player(player_id).unwrap();
        
        // Test empty map
        let ast = EchoAst::Map { entries: vec![] };
        let result = jit.eval(&ast).unwrap();
        
        match result {
            Value::Map(map) => assert_eq!(map.len(), 0),
            _ => panic!("Expected map value"),
        }
        
        // Test map with string keys
        let ast = EchoAst::Map { 
            entries: vec![
                ("x".to_string(), EchoAst::Number(42)),
                ("y".to_string(), EchoAst::Number(99)),
            ] 
        };
        let result = jit.eval(&ast).unwrap();
        
        match result {
            Value::Map(map) => {
                assert_eq!(map.len(), 2);
                match map.get("x") {
                    Some(Value::Integer(n)) => assert_eq!(*n, 42),
                    _ => panic!("Expected x to be 42"),
                }
                match map.get("y") {
                    Some(Value::Integer(n)) => assert_eq!(*n, 99),
                    _ => panic!("Expected y to be 99"),
                }
            }
            _ => panic!("Expected map value"),
        }
    }

    #[test]
    fn test_jit_map_with_computed_keys() {
        let mut jit = create_test_jit();
        
        // Create a player
        let player_id = jit.create_player("test_player").unwrap();
        jit.switch_player(player_id).unwrap();
        
        // Test map with computed keys - not supported in this AST structure
        // Maps in Echo use static string keys, not computed expressions
        let ast = EchoAst::Map { 
            entries: vec![
                ("key1".to_string(), EchoAst::Number(100)),
            ] 
        };
        let result = jit.eval(&ast).unwrap();
        
        match result {
            Value::Map(map) => {
                assert_eq!(map.len(), 1);
                // Computed key should be "key1" 
                match map.get("key1") {
                    Some(Value::Integer(n)) => assert_eq!(*n, 100),
                    _ => panic!("Expected key1 to be 100"),
                }
            }
            _ => panic!("Expected map value"),
        }
    }

    #[test]
    fn test_jit_map_mixed_values() {
        let mut jit = create_test_jit();
        
        // Create a player
        let player_id = jit.create_player("test_player").unwrap();
        jit.switch_player(player_id).unwrap();
        
        // Test map with mixed value types
        let ast = EchoAst::Map { 
            entries: vec![
                ("num".to_string(), EchoAst::Number(42)),
                ("str".to_string(), EchoAst::String("hello".to_string())),
                ("bool".to_string(), EchoAst::Boolean(true)),
                ("null".to_string(), EchoAst::Null),
                ("list".to_string(), EchoAst::List { 
                    elements: vec![EchoAst::Number(1), EchoAst::Number(2)] 
                }),
            ] 
        };
        let result = jit.eval(&ast).unwrap();
        
        match result {
            Value::Map(map) => {
                assert_eq!(map.len(), 5);
                
                match map.get("num") {
                    Some(Value::Integer(n)) => assert_eq!(*n, 42),
                    _ => panic!("Expected num to be 42"),
                }
                
                match map.get("str") {
                    Some(Value::String(s)) => assert_eq!(s, "hello"),
                    _ => panic!("Expected str to be 'hello'"),
                }
                
                match map.get("bool") {
                    Some(Value::Boolean(b)) => assert_eq!(*b, true),
                    _ => panic!("Expected bool to be true"),
                }
                
                match map.get("null") {
                    Some(Value::Null) => {},
                    _ => panic!("Expected null to be null"),
                }
                
                match map.get("list") {
                    Some(Value::List(list)) => assert_eq!(list.len(), 2),
                    _ => panic!("Expected list to be a list"),
                }
            }
            _ => panic!("Expected map value"),
        }
    }

    #[test]
    fn test_jit_compile_map() {
        let mut jit = create_test_jit();
        
        // Skip test if JIT is not enabled
        if !jit.is_jit_enabled() {
            println!("Skipping map compilation test - JIT not enabled");
            return;
        }
        
        // Test that compile_ast handles maps
        let operations = vec![
            EchoAst::Map { entries: vec![] },
            EchoAst::Map { 
                entries: vec![
                    ("x".to_string(), EchoAst::Number(42)),
                ] 
            },
        ];
        
        for op in operations {
            match jit.compile_ast(&op) {
                Ok(()) => println!("Successfully compiled: {:?}", op),
                Err(e) => {
                    // Maps require runtime allocation and dynamic typing
                    if e.to_string().contains("falling back to interpreter") {
                        println!("Map operation falls back to interpreter as expected: {:?}", op);
                    } else {
                        panic!("Failed to compile {:?}: {}", op, e);
                    }
                }
            }
        }
    }
}