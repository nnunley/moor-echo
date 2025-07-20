// Tests for object reference resolution with object_map

#[cfg(test)]
mod tests {
    use crate::evaluator::{create_evaluator, Value};
    use crate::parser::create_parser;
    use crate::storage::{Storage, ObjectId};
    use std::sync::Arc;
    use tempfile::tempdir;

    #[test]
    fn test_object_ref_with_map_property() {
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let storage = Arc::new(Storage::new(temp_dir.path()).expect("Failed to create storage"));
        let mut evaluator = create_evaluator(storage).expect("Failed to create evaluator");
        
        // Create a player
        let player_id = evaluator.create_player("test_player").expect("Failed to create player");
        evaluator.switch_player(player_id).expect("Failed to switch player");
        
        // Create some objects
        let mut parser = create_parser("echo").expect("Failed to create parser");
        let code = r#"
            object foo
                property name = "Foo Object"
            endobject
            
            object bar
                property name = "Bar Object"
            endobject
        "#;
        
        let ast = parser.parse(code).expect("Failed to parse");
        evaluator.eval(&ast).expect("Failed to evaluate");
        
        // Set up object_map
        let map_code = r#"
            #0.object_map = {"10": #0.foo, "20": #0.bar}
        "#;
        
        let map_ast = parser.parse(map_code).expect("Failed to parse");
        evaluator.eval(&map_ast).expect("Failed to evaluate");
        
        // Test numeric references
        let ref10_ast = parser.parse("#10").expect("Failed to parse");
        let result10 = evaluator.eval(&ref10_ast).expect("Failed to evaluate");
        
        // Verify we got the foo object
        let name_code = "#10.name";
        let name_ast = parser.parse(name_code).expect("Failed to parse");
        let name_result = evaluator.eval(&name_ast).expect("Failed to evaluate");
        assert_eq!(name_result, Value::String("Foo Object".to_string()));
        
        // Test another reference
        let ref20_ast = parser.parse("#20").expect("Failed to parse");
        let result20 = evaluator.eval(&ref20_ast).expect("Failed to evaluate");
        
        let name_code = "#20.name";
        let name_ast = parser.parse(name_code).expect("Failed to parse");
        let name_result = evaluator.eval(&name_ast).expect("Failed to evaluate");
        assert_eq!(name_result, Value::String("Bar Object".to_string()));
    }
    
    #[test]
    fn test_object_ref_with_verb() {
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let storage = Arc::new(Storage::new(temp_dir.path()).expect("Failed to create storage"));
        let mut evaluator = create_evaluator(storage).expect("Failed to create evaluator");
        
        // Create a player
        let player_id = evaluator.create_player("test_player").expect("Failed to create player");
        evaluator.switch_player(player_id).expect("Failed to switch player");
        
        // Create objects and define object_map verb
        let mut parser = create_parser("echo").expect("Failed to create parser");
        let code = r#"
            object test_obj
                property name = "Test Object"
            endobject
            
            object #0
                verb object_map {n}
                    if n == 100
                        return #0.test_obj
                    else
                        return null
                    endif
                endverb
            endobject
        "#;
        
        let ast = parser.parse(code).expect("Failed to parse");
        evaluator.eval(&ast).expect("Failed to evaluate");
        
        // Test numeric reference that should resolve
        let ref100_ast = parser.parse("#100").expect("Failed to parse");
        let result100 = evaluator.eval(&ref100_ast).expect("Failed to evaluate");
        
        // Verify we got the test_obj
        let name_code = "#100.name";
        let name_ast = parser.parse(name_code).expect("Failed to parse");
        let name_result = evaluator.eval(&name_ast).expect("Failed to evaluate");
        assert_eq!(name_result, Value::String("Test Object".to_string()));
        
        // Test numeric reference that should fail
        let ref200_ast = parser.parse("#200").expect("Failed to parse");
        let result200 = evaluator.eval(&ref200_ast);
        assert!(result200.is_err());
        assert!(result200.unwrap_err().to_string().contains("Object reference #200 not found"));
    }
    
    #[test]
    fn test_builtin_object_refs() {
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let storage = Arc::new(Storage::new(temp_dir.path()).expect("Failed to create storage"));
        let mut evaluator = create_evaluator(storage).expect("Failed to create evaluator");
        
        // Create a player
        let player_id = evaluator.create_player("test_player").expect("Failed to create player");
        evaluator.switch_player(player_id).expect("Failed to switch player");
        
        let mut parser = create_parser("echo").expect("Failed to create parser");
        
        // Test #0 (system object)
        let ref0_ast = parser.parse("#0").expect("Failed to parse");
        let result0 = evaluator.eval(&ref0_ast).expect("Failed to evaluate");
        assert_eq!(result0, Value::Object(ObjectId::system()));
        
        // Test #1 (root object)
        let ref1_ast = parser.parse("#1").expect("Failed to parse");
        let result1 = evaluator.eval(&ref1_ast).expect("Failed to evaluate");
        assert_eq!(result1, Value::Object(ObjectId::root()));
    }
    
    #[test]
    fn test_object_ref_error_message() {
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let storage = Arc::new(Storage::new(temp_dir.path()).expect("Failed to create storage"));
        let mut evaluator = create_evaluator(storage).expect("Failed to create evaluator");
        
        // Create a player
        let player_id = evaluator.create_player("test_player").expect("Failed to create player");
        evaluator.switch_player(player_id).expect("Failed to switch player");
        
        let mut parser = create_parser("echo").expect("Failed to create parser");
        
        // Test unmapped reference
        let ref99_ast = parser.parse("#99").expect("Failed to parse");
        let result99 = evaluator.eval(&ref99_ast);
        
        assert!(result99.is_err());
        let error_msg = result99.unwrap_err().to_string();
        assert!(error_msg.contains("Object reference #99 not found"));
        assert!(error_msg.contains("objects are typically referenced by name"));
        assert!(error_msg.contains("Define #0:object_map(n)"));
        assert!(error_msg.contains("Set #0.object_map as a map"));
    }
}