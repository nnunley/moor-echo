use echo_repl::parser::create_parser;
use echo_repl::evaluator::{create_evaluator, Value};
use echo_repl::storage::Storage;
use std::sync::Arc;
use tempfile::tempdir;

// Helper function to parse Echo code
fn parse_echo(code: &str) -> anyhow::Result<echo_repl::ast::EchoAst> {
    let mut parser = create_parser("echo")?;
    parser.parse(code)
}

#[test]
fn test_simple_for_loop() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let storage = Arc::new(Storage::new(temp_dir.path()).expect("Failed to create storage"));
    let mut evaluator = create_evaluator(storage).expect("Failed to create evaluator");
    
    // Create a player
    let player_id = evaluator.create_player("test_player").expect("Failed to create player");
    evaluator.switch_player(player_id).expect("Failed to switch player");
    
    // Test for loop: for (item in [1, 2, 3]) return item; endfor
    // This should return the last item (3)
    let mut parser = create_parser("echo").expect("Failed to create parser");
    let ast = parser.parse("for (item in [1, 2, 3]) return item; endfor").expect("Failed to parse");
    let result = evaluator.eval(&ast).expect("Failed to evaluate");
    
    assert_eq!(result, Value::Integer(3));
}

#[test]
fn test_for_loop_with_string_iteration() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let storage = Arc::new(Storage::new(temp_dir.path()).expect("Failed to create storage"));
    let mut evaluator = create_evaluator(storage).expect("Failed to create evaluator");
    
    // Create a player
    let player_id = evaluator.create_player("test_player").expect("Failed to create player");
    evaluator.switch_player(player_id).expect("Failed to switch player");
    
    // Test for loop with string: for (char in "abc") return char; endfor
    // This should return the last character "c"
    let ast = parse_echo("for (char in \"abc\") return char; endfor").expect("Failed to parse");
    let result = evaluator.eval(&ast).expect("Failed to evaluate");
    
    assert_eq!(result, Value::String("c".to_string()));
}

#[test]
fn test_for_loop_with_accumulator() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let storage = Arc::new(Storage::new(temp_dir.path()).expect("Failed to create storage"));
    let mut evaluator = create_evaluator(storage).expect("Failed to create evaluator");
    
    // Create a player
    let player_id = evaluator.create_player("test_player").expect("Failed to create player");
    evaluator.switch_player(player_id).expect("Failed to switch player");
    
    // Initialize accumulator
    let init_ast = parse_echo("let sum = 0;").expect("Failed to parse init");
    evaluator.eval(&init_ast).expect("Failed to evaluate init");
    
    // Test for loop: for (item in [1, 2, 3]) let sum = sum + item; endfor
    let ast = parse_echo("for (item in [1, 2, 3]) let sum = sum + item; endfor").expect("Failed to parse");
    let result = evaluator.eval(&ast).expect("Failed to evaluate");
    
    // Should return the last assignment (sum = 6)
    assert_eq!(result, Value::Integer(6));
    
    // Verify sum variable is updated
    let sum_check = parse_echo("sum").expect("Failed to parse sum check");
    let sum_result = evaluator.eval(&sum_check).expect("Failed to evaluate sum check");
    assert_eq!(sum_result, Value::Integer(6));
}

#[test]
fn test_while_loop() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let storage = Arc::new(Storage::new(temp_dir.path()).expect("Failed to create storage"));
    let mut evaluator = create_evaluator(storage).expect("Failed to create evaluator");
    
    // Create a player
    let player_id = evaluator.create_player("test_player").expect("Failed to create player");
    evaluator.switch_player(player_id).expect("Failed to switch player");
    
    // Initialize counter
    let init_ast = parse_echo("let i = 0;").expect("Failed to parse init");
    evaluator.eval(&init_ast).expect("Failed to evaluate init");
    
    // Test while loop: while (i < 3) let i = i + 1; endwhile
    let ast = parse_echo("while (i < 3) let i = i + 1; endwhile").expect("Failed to parse");
    let result = evaluator.eval(&ast).expect("Failed to evaluate");
    
    // Should return the last assignment (i = 3)
    assert_eq!(result, Value::Integer(3));
    
    // Verify counter variable is updated
    let i_check = parse_echo("i").expect("Failed to parse i check");
    let i_result = evaluator.eval(&i_check).expect("Failed to evaluate i check");
    assert_eq!(i_result, Value::Integer(3));
}

#[test]
fn test_for_loop_with_break() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let storage = Arc::new(Storage::new(temp_dir.path()).expect("Failed to create storage"));
    let mut evaluator = create_evaluator(storage).expect("Failed to create evaluator");
    
    // Create a player
    let player_id = evaluator.create_player("test_player").expect("Failed to create player");
    evaluator.switch_player(player_id).expect("Failed to switch player");
    
    // Initialize result
    let init_ast = parse_echo("let result = 0;").expect("Failed to parse init");
    evaluator.eval(&init_ast).expect("Failed to evaluate init");
    
    // Test for loop with break: for (item in [1, 2, 3, 4, 5]) if (item == 3) break; endif let result = item; endfor
    let ast = parse_echo("for (item in [1, 2, 3, 4, 5]) if (item == 3) break; endif let result = item; endfor").expect("Failed to parse");
    let result = evaluator.eval(&ast).expect("Failed to evaluate");
    
    // Should return the last assignment before break (result = 2)
    assert_eq!(result, Value::Integer(2));
    
    // Verify result variable
    let result_check = parse_echo("result").expect("Failed to parse result check");
    let result_val = evaluator.eval(&result_check).expect("Failed to evaluate result check");
    assert_eq!(result_val, Value::Integer(2));
}

#[test]
fn test_for_loop_with_continue() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let storage = Arc::new(Storage::new(temp_dir.path()).expect("Failed to create storage"));
    let mut evaluator = create_evaluator(storage).expect("Failed to create evaluator");
    
    // Create a player
    let player_id = evaluator.create_player("test_player").expect("Failed to create player");
    evaluator.switch_player(player_id).expect("Failed to switch player");
    
    // Initialize result
    let init_ast = parse_echo("let result = 0;").expect("Failed to parse init");
    evaluator.eval(&init_ast).expect("Failed to evaluate init");
    
    // Test for loop with continue: for (item in [1, 2, 3, 4, 5]) if (item == 3) continue; endif let result = item; endfor
    let ast = parse_echo("for (item in [1, 2, 3, 4, 5]) if (item == 3) continue; endif let result = item; endfor").expect("Failed to parse");
    let result = evaluator.eval(&ast).expect("Failed to evaluate");
    
    // Should return the last assignment (result = 5, skipping 3)
    assert_eq!(result, Value::Integer(5));
    
    // Verify result variable
    let result_check = parse_echo("result").expect("Failed to parse result check");
    let result_val = evaluator.eval(&result_check).expect("Failed to evaluate result check");
    assert_eq!(result_val, Value::Integer(5));
}

#[test]
fn test_while_loop_with_break() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let storage = Arc::new(Storage::new(temp_dir.path()).expect("Failed to create storage"));
    let mut evaluator = create_evaluator(storage).expect("Failed to create evaluator");
    
    // Create a player
    let player_id = evaluator.create_player("test_player").expect("Failed to create player");
    evaluator.switch_player(player_id).expect("Failed to switch player");
    
    // Initialize counter
    let init_ast = parse_echo("let i = 0;").expect("Failed to parse init");
    evaluator.eval(&init_ast).expect("Failed to evaluate init");
    
    // Test while loop with break: while (true) let i = i + 1; if (i == 3) break; endif endwhile
    let ast = parse_echo("while (true) let i = i + 1; if (i == 3) break; endif endwhile").expect("Failed to parse");
    let result = evaluator.eval(&ast).expect("Failed to evaluate");
    
    // Should return the last assignment (i = 3)
    assert_eq!(result, Value::Integer(3));
    
    // Verify counter variable
    let i_check = parse_echo("i").expect("Failed to parse i check");
    let i_result = evaluator.eval(&i_check).expect("Failed to evaluate i check");
    assert_eq!(i_result, Value::Integer(3));
}

#[test]
fn test_nested_loops() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let storage = Arc::new(Storage::new(temp_dir.path()).expect("Failed to create storage"));
    let mut evaluator = create_evaluator(storage).expect("Failed to create evaluator");
    
    // Create a player
    let player_id = evaluator.create_player("test_player").expect("Failed to create player");
    evaluator.switch_player(player_id).expect("Failed to switch player");
    
    // Initialize result
    let init_ast = parse_echo("let result = 0;").expect("Failed to parse init");
    evaluator.eval(&init_ast).expect("Failed to evaluate init");
    
    // Test nested loops: for (i in [1, 2]) for (j in [3, 4]) let result = i + j; endfor endfor
    let ast = parse_echo("for (i in [1, 2]) for (j in [3, 4]) let result = i + j; endfor endfor").expect("Failed to parse");
    let result = evaluator.eval(&ast).expect("Failed to evaluate");
    
    // Should return the last assignment (result = 2 + 4 = 6)
    assert_eq!(result, Value::Integer(6));
    
    // Verify result variable
    let result_check = parse_echo("result").expect("Failed to parse result check");
    let result_val = evaluator.eval(&result_check).expect("Failed to evaluate result check");
    assert_eq!(result_val, Value::Integer(6));
}

#[test]
fn test_empty_list_iteration() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let storage = Arc::new(Storage::new(temp_dir.path()).expect("Failed to create storage"));
    let mut evaluator = create_evaluator(storage).expect("Failed to create evaluator");
    
    // Create a player
    let player_id = evaluator.create_player("test_player").expect("Failed to create player");
    evaluator.switch_player(player_id).expect("Failed to switch player");
    
    // Test for loop with empty list: for (item in []) return item; endfor
    let ast = parse_echo("for (item in []) return item; endfor").expect("Failed to parse");
    let result = evaluator.eval(&ast).expect("Failed to evaluate");
    
    // Should return null (no iterations)
    assert_eq!(result, Value::Null);
}

#[test]
fn test_list_literal_evaluation() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let storage = Arc::new(Storage::new(temp_dir.path()).expect("Failed to create storage"));
    let mut evaluator = create_evaluator(storage).expect("Failed to create evaluator");
    
    // Create a player
    let player_id = evaluator.create_player("test_player").expect("Failed to create player");
    evaluator.switch_player(player_id).expect("Failed to switch player");
    
    // Test list literal: [1, 2, 3]
    let ast = parse_echo("[1, 2, 3]").expect("Failed to parse");
    let result = evaluator.eval(&ast).expect("Failed to evaluate");
    
    assert_eq!(result, Value::List(vec![
        Value::Integer(1),
        Value::Integer(2),
        Value::Integer(3)
    ]));
}

#[test]
fn test_mixed_type_list_evaluation() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let storage = Arc::new(Storage::new(temp_dir.path()).expect("Failed to create storage"));
    let mut evaluator = create_evaluator(storage).expect("Failed to create evaluator");
    
    // Create a player
    let player_id = evaluator.create_player("test_player").expect("Failed to create player");
    evaluator.switch_player(player_id).expect("Failed to switch player");
    
    // Test mixed type list: [1, "hello", true]
    let ast = parse_echo("[1, \"hello\", true]").expect("Failed to parse");
    let result = evaluator.eval(&ast).expect("Failed to evaluate");
    
    assert_eq!(result, Value::List(vec![
        Value::Integer(1),
        Value::String("hello".to_string()),
        Value::Boolean(true)
    ]));
}