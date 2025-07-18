use echo_repl::parser::{EchoParser};
use echo_repl::evaluator::{Evaluator, Value};
use echo_repl::storage::Storage;
use std::sync::Arc;

#[test]
fn test_string_literal() {
    let mut parser = EchoParser::new().unwrap();
    let ast = parser.parse(r#""hello world""#).unwrap();
    
    let storage = Arc::new(Storage::new("./test-string-db").unwrap());
    let mut evaluator = Evaluator::new(storage);
    let player_id = evaluator.create_player("test").unwrap();
    
    let result = evaluator.eval_with_player(&ast, player_id).unwrap();
    assert_eq!(result, Value::String("hello world".to_string()));
}

#[test]
fn test_string_with_let() {
    let mut parser = EchoParser::new().unwrap();
    let ast = parser.parse(r#"let message = "hello rust-sitter";"#).unwrap();
    
    let storage = Arc::new(Storage::new("./test-string-let-db").unwrap());
    let mut evaluator = Evaluator::new(storage);
    let player_id = evaluator.create_player("test").unwrap();
    
    let result = evaluator.eval_with_player(&ast, player_id).unwrap();
    assert_eq!(result, Value::String("hello rust-sitter".to_string()));
    
    // Test that the variable was stored
    let var_ast = parser.parse("message").unwrap();
    let var_result = evaluator.eval_with_player(&var_ast, player_id).unwrap();
    assert_eq!(var_result, Value::String("hello rust-sitter".to_string()));
}