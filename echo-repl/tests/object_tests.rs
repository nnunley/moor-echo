use echo_repl::parser::EchoParser;
use echo_repl::evaluator::{Evaluator, Value};
use echo_repl::storage::Storage;
use std::sync::Arc;

#[test]
fn test_object_definition() {
    let mut parser = EchoParser::new().unwrap();
    let ast = parser.parse(r#"object testobj
    property name = "TestObject";
    property value = 42;
endobject"#).unwrap();
    
    let storage = Arc::new(Storage::new("./test-object-db").unwrap());
    let mut evaluator = Evaluator::new(storage.clone());
    let player_id = evaluator.create_player("test").unwrap();
    
    let result = evaluator.eval_with_player(&ast, player_id).unwrap();
    
    // Should return an object ID
    if let Value::Object(obj_id) = result {
        // Verify object was stored
        let obj = storage.objects.get(obj_id).unwrap();
        assert_eq!(obj.name, "testobj");
        
        // Verify properties were set
        assert_eq!(obj.properties.len(), 2);
        assert!(obj.properties.contains_key("name"));
        assert!(obj.properties.contains_key("value"));
    } else {
        panic!("Expected object result, got {:?}", result);
    }
}

#[test]
fn test_object_property_access() {
    let mut parser = EchoParser::new().unwrap();
    
    // Create object
    let obj_ast = parser.parse(r#"object testobj
    property name = "TestObject";
    property value = 42;
endobject"#).unwrap();
    
    let storage = Arc::new(Storage::new("./test-object-prop-db").unwrap());
    let mut evaluator = Evaluator::new(storage);
    let player_id = evaluator.create_player("test").unwrap();
    
    evaluator.eval_with_player(&obj_ast, player_id).unwrap();
    
    // Test property access
    let prop_ast = parser.parse("testobj.name").unwrap();
    let result = evaluator.eval_with_player(&prop_ast, player_id).unwrap();
    
    assert_eq!(result, Value::String("TestObject".to_string()));
    
    // Test numeric property access
    let num_prop_ast = parser.parse("testobj.value").unwrap();
    let num_result = evaluator.eval_with_player(&num_prop_ast, player_id).unwrap();
    
    assert_eq!(num_result, Value::Integer(42));
}