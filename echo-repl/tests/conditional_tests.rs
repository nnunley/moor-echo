use echo_repl::parser::{create_parser, Parser};
use echo_repl::evaluator::{create_evaluator, Value};
use echo_repl::storage::Storage;
use std::sync::Arc;
use tempfile::tempdir;

#[test]
fn test_conditional_true_branch() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let storage = Arc::new(Storage::new(temp_dir.path()).expect("Failed to create storage"));
    let mut evaluator = create_evaluator(storage).expect("Failed to create evaluator");
    
    // Create a player
    let player_id = evaluator.create_player("test_player").expect("Failed to create player");
    evaluator.switch_player(player_id).expect("Failed to switch player");
    
    // Test if (true) return 42; endif
    let mut parser = create_parser("echo").expect("Failed to create parser");
    let ast = parser.parse("if (true) return 42; endif").expect("Failed to parse");
    let result = evaluator.eval(&ast).expect("Failed to evaluate");
    
    assert_eq!(result, Value::Integer(42));
}

#[test]
fn test_conditional_false_branch() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let storage = Arc::new(Storage::new(temp_dir.path()).expect("Failed to create storage"));
    let mut evaluator = create_evaluator(storage).expect("Failed to create evaluator");
    
    // Create a player
    let player_id = evaluator.create_player("test_player").expect("Failed to create player");
    evaluator.switch_player(player_id).expect("Failed to switch player");
    
    // Test if (false) return 42; endif (should return null)
    let mut parser = create_parser("echo").expect("Failed to create parser");
    let ast = parser.parse("if (false) return 42; endif").expect("Failed to parse");
    let result = evaluator.eval(&ast).expect("Failed to evaluate");
    
    assert_eq!(result, Value::Null);
}

#[test]
fn test_conditional_if_else() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let storage = Arc::new(Storage::new(temp_dir.path()).expect("Failed to create storage"));
    let mut evaluator = create_evaluator(storage).expect("Failed to create evaluator");
    
    // Create a player
    let player_id = evaluator.create_player("test_player").expect("Failed to create player");
    evaluator.switch_player(player_id).expect("Failed to switch player");
    
    // Test if (false) return 1; else return 2; endif
    let mut parser = create_parser("echo").expect("Failed to create parser");
    let ast = parser.parse("if (false) return 1; else return 2; endif").expect("Failed to parse");
    let result = evaluator.eval(&ast).expect("Failed to evaluate");
    
    assert_eq!(result, Value::Integer(2));
}

#[test]
fn test_conditional_with_comparison() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let storage = Arc::new(Storage::new(temp_dir.path()).expect("Failed to create storage"));
    let mut evaluator = create_evaluator(storage).expect("Failed to create evaluator");
    
    // Create a player
    let player_id = evaluator.create_player("test_player").expect("Failed to create player");
    evaluator.switch_player(player_id).expect("Failed to switch player");
    
    // Test if (1 == 1) return "equal"; else return "not equal"; endif
    let mut parser = create_parser("echo").expect("Failed to create parser");
    let ast = parser.parse("if (1 == 1) return \"equal\"; else return \"not equal\"; endif").expect("Failed to parse");
    let result = evaluator.eval(&ast).expect("Failed to evaluate");
    
    assert_eq!(result, Value::String("equal".to_string()));
}

#[test]
fn test_comparison_operators() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let storage = Arc::new(Storage::new(temp_dir.path()).expect("Failed to create storage"));
    let mut evaluator = create_evaluator(storage).expect("Failed to create evaluator");
    
    // Create a player
    let player_id = evaluator.create_player("test_player").expect("Failed to create player");
    evaluator.switch_player(player_id).expect("Failed to switch player");
    
    // Test various comparison operators
    let test_cases = vec![
        ("1 == 1", Value::Boolean(true)),
        ("1 == 2", Value::Boolean(false)),
        ("1 != 2", Value::Boolean(true)),
        ("1 != 1", Value::Boolean(false)),
        ("1 < 2", Value::Boolean(true)),
        ("2 < 1", Value::Boolean(false)),
        ("2 > 1", Value::Boolean(true)),
        ("1 > 2", Value::Boolean(false)),
        ("1 <= 1", Value::Boolean(true)),
        ("1 <= 2", Value::Boolean(true)),
        ("2 <= 1", Value::Boolean(false)),
        ("2 >= 2", Value::Boolean(true)),
        ("2 >= 1", Value::Boolean(true)),
        ("1 >= 2", Value::Boolean(false)),
    ];
    
    for (expr, expected) in test_cases {
        let mut parser = create_parser("echo").expect("Failed to create parser");
        let ast = parser.parse(expr).expect(&format!("Failed to parse: {}", expr));
        let result = evaluator.eval(&ast).expect(&format!("Failed to evaluate: {}", expr));
        assert_eq!(result, expected, "Failed for expression: {}", expr);
    }
}