use echo_repl::parser::EchoParser;
use echo_repl::evaluator::{Evaluator, Value};
use echo_repl::storage::Storage;
use std::sync::Arc;

#[test]
fn test_method_call() {
    let mut parser = EchoParser::new().unwrap();
    
    // Create object with verb
    let obj_ast = parser.parse(r#"object testobj
    property greeting = "Hello";
    verb "greet" (this, "none", "none")
        return "Hello World!";
    endverb
endobject"#).unwrap();
    
    let storage = Arc::new(Storage::new("./test-method-db").unwrap());
    let mut evaluator = Evaluator::new(storage);
    let player_id = evaluator.create_player("test").unwrap();
    
    evaluator.eval_with_player(&obj_ast, player_id).unwrap();
    
    // Test method call
    let call_ast = parser.parse("testobj:greet()").unwrap();
    let result = evaluator.eval_with_player(&call_ast, player_id).unwrap();
    
    // Check that we get the expected result from the greet method
    assert_eq!(result, Value::String("Hello World!".to_string()));
}

#[test]
fn test_method_call_undefined() {
    let mut parser = EchoParser::new().unwrap();
    
    // Create object without verb
    let obj_ast = parser.parse(r#"object testobj
    property greeting = "Hello";
endobject"#).unwrap();
    
    let storage = Arc::new(Storage::new("./test-method-undef-db").unwrap());
    let mut evaluator = Evaluator::new(storage);
    let player_id = evaluator.create_player("test").unwrap();
    
    evaluator.eval_with_player(&obj_ast, player_id).unwrap();
    
    // Test method call on undefined method
    let call_ast = parser.parse("testobj:nonexistent()").unwrap();
    let result = evaluator.eval_with_player(&call_ast, player_id);
    
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Method 'nonexistent' not found"));
}