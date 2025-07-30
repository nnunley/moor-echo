//! Tests for JIT compilation of reference operations (SystemProperty, ObjectRef)

#[cfg(all(test, feature = "jit"))]
mod tests {
    use super::super::*;
    use crate::ast::EchoAst;
    use crate::storage::{Storage, PropertyValue, ObjectId};
    use crate::evaluator::value_to_property_value;
    use std::sync::Arc;
    use std::collections::HashMap;
    use tempfile::TempDir;

    fn create_test_jit() -> JitEvaluator {
        let temp_dir = TempDir::new().unwrap();
        let storage = Arc::new(Storage::new(temp_dir.path()).unwrap());
        JitEvaluator::new_with_fallback(storage)
    }

    #[test]
    fn test_jit_system_property() {
        let mut jit = create_test_jit();
        
        // Create a player
        let player_id = jit.create_player("test_player").unwrap();
        jit.switch_player(player_id).unwrap();
        
        // Set up a system property
        // Set up a system property
        let system_obj_id = ObjectId::system();
        let mut system_obj = jit.storage().objects.get(system_obj_id).unwrap();
        
        // Add a property to system object
        system_obj.properties.insert("test_prop".to_string(), PropertyValue::String("test_value".to_string()));
        jit.storage().objects.store(system_obj).unwrap();
        
        // Test $test_prop
        let ast = EchoAst::SystemProperty("test_prop".to_string());
        let result = jit.eval(&ast).unwrap();
        
        match result {
            Value::String(s) => assert_eq!(s, "test_value"),
            _ => panic!("Expected string value"),
        }
    }

    #[test]
    fn test_jit_object_ref() {
        let mut jit = create_test_jit();
        
        // Create a player
        let player_id = jit.create_player("test_player").unwrap();
        jit.switch_player(player_id).unwrap();
        
        // Test #0 (system object)
        let ast = EchoAst::ObjectRef(0);
        let result = jit.eval(&ast).unwrap();
        
        match result {
            Value::Object(_) => {}, // Just verify it's an object
            _ => panic!("Expected object value"),
        }
        
        // Test #1 (root object)
        let ast = EchoAst::ObjectRef(1);
        let result = jit.eval(&ast).unwrap();
        
        match result {
            Value::Object(_) => {}, // Just verify it's an object
            _ => panic!("Expected object value"),
        }
    }

    #[test]
    fn test_jit_object_ref_with_map() {
        let mut jit = create_test_jit();
        
        // Create a player
        let player_id = jit.create_player("test_player").unwrap();
        jit.switch_player(player_id).unwrap();
        
        // Create an object_map on system object
        let system_obj_id = ObjectId::system();
        let mut system_obj = jit.storage().objects.get(system_obj_id).unwrap();
        
        // Create a map with object references (maps use string keys)
        let mut map_entries = HashMap::new();
        map_entries.insert("42".to_string(), Value::String("object_42".to_string()));
        map_entries.insert("99".to_string(), Value::String("object_99".to_string()));
        let object_map = Value::Map(map_entries);
        system_obj.properties.insert("object_map".to_string(), value_to_property_value(object_map).unwrap());
        jit.storage().objects.store(system_obj).unwrap();
        
        // Test #42 (should look up in object_map)
        let ast = EchoAst::ObjectRef(42);
        let result = jit.eval(&ast).unwrap();
        
        match result {
            Value::String(s) => assert_eq!(s, "object_42"),
            _ => panic!("Expected string value"),
        }
    }

    #[test]
    fn test_jit_compile_references() {
        let mut jit = create_test_jit();
        
        // Skip test if JIT is not enabled
        if !jit.is_jit_enabled() {
            println!("Skipping reference compilation test - JIT not enabled");
            return;
        }
        
        // Test that compile_ast handles reference operations
        let operations = vec![
            EchoAst::SystemProperty("test".to_string()),
            EchoAst::ObjectRef(0),
            EchoAst::ObjectRef(123),
        ];
        
        for op in operations {
            match jit.compile_ast(&op) {
                Ok(()) => println!("Successfully compiled: {:?}", op),
                Err(e) => {
                    // Reference operations require runtime lookup
                    if e.to_string().contains("falling back to interpreter") {
                        println!("Reference operation falls back to interpreter as expected: {:?}", op);
                    } else {
                        panic!("Failed to compile {:?}: {}", op, e);
                    }
                }
            }
        }
    }
}