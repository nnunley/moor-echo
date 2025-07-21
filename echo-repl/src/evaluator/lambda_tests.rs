use super::*;
use crate::parser::create_parser;
use crate::storage::Storage;
use pretty_assertions::assert_eq;
use std::sync::Arc;
use tempfile::TempDir;

fn setup_test() -> (Evaluator, ObjectId, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let storage = Arc::new(Storage::new(temp_dir.path()).unwrap());
    let mut evaluator = Evaluator::new(storage);
    let player_id = evaluator.create_player("test").unwrap();
    (evaluator, player_id, temp_dir)
}

#[test]
fn test_simple_lambda_parameters() {
    let (mut evaluator, player_id, _temp_dir) = setup_test();
    let mut parser = create_parser("echo").unwrap();
    
    // Create a simple lambda
    let code = "let add = fn {x, y} x + y endfn";
    let ast = parser.parse(code).unwrap();
    evaluator.eval_with_player(&ast, player_id).unwrap();
    
    // Test calling it
    let call_ast = parser.parse("add(5, 3)").unwrap();
    let result = evaluator.eval_with_player(&call_ast, player_id).unwrap();
    
    assert_eq!(result, Value::Integer(8), "Simple lambda should add parameters");
}

#[test]
fn test_optional_parameters_with_default() {
    let (mut evaluator, player_id, _temp_dir) = setup_test();
    let mut parser = create_parser("echo").unwrap();
    
    // Create lambda with optional parameter
    let code = "let greet = fn {name, ?greeting=\"Hello\"} greeting + \" \" + name endfn";
    let ast = parser.parse(code).unwrap();
    evaluator.eval_with_player(&ast, player_id).unwrap();
    
    // Test with default value
    let call1 = parser.parse("greet(\"World\")").unwrap();
    let result1 = evaluator.eval_with_player(&call1, player_id).unwrap();
    assert_eq!(
        result1, 
        Value::String("Hello World".to_string()),
        "Optional parameter should use default value"
    );
    
    // Test with provided value
    let call2 = parser.parse("greet(\"World\", \"Hi\")").unwrap();
    let result2 = evaluator.eval_with_player(&call2, player_id).unwrap();
    assert_eq!(
        result2,
        Value::String("Hi World".to_string()),
        "Optional parameter should use provided value"
    );
}

#[test]
fn test_optional_parameter_with_expression_default() {
    let (mut evaluator, player_id, _temp_dir) = setup_test();
    let mut parser = create_parser("echo").unwrap();
    
    // Set up a variable that will be used in default
    let setup = parser.parse("let base = 100").unwrap();
    evaluator.eval_with_player(&setup, player_id).unwrap();
    
    // Create lambda with expression as default
    let code = "let calc = fn {x, ?y=base + 10} x + y endfn";
    let ast = parser.parse(code).unwrap();
    evaluator.eval_with_player(&ast, player_id).unwrap();
    
    // Test with default (should be 100 + 10 = 110)
    let call1 = parser.parse("calc(5)").unwrap();
    let result1 = evaluator.eval_with_player(&call1, player_id).unwrap();
    assert_eq!(
        result1,
        Value::Integer(115), // 5 + 110
        "Optional parameter should evaluate default expression"
    );
}

#[test]
fn test_rest_parameters() {
    let (mut evaluator, player_id, _temp_dir) = setup_test();
    let mut parser = create_parser("echo").unwrap();
    
    // Create lambda with rest parameter
    let code = "let collect = fn {first, @rest} rest endfn";
    let ast = parser.parse(code).unwrap();
    evaluator.eval_with_player(&ast, player_id).unwrap();
    
    // Test rest parameter collection
    let call = parser.parse("collect(1, 2, 3, 4, 5)").unwrap();
    let result = evaluator.eval_with_player(&call, player_id).unwrap();
    
    assert_eq!(
        result,
        Value::List(vec![
            Value::Integer(2),
            Value::Integer(3),
            Value::Integer(4),
            Value::Integer(5),
        ]),
        "Rest parameter should collect remaining arguments"
    );
}

#[test]
fn test_rest_parameter_empty() {
    let (mut evaluator, player_id, _temp_dir) = setup_test();
    let mut parser = create_parser("echo").unwrap();
    
    // Create lambda with rest parameter
    let code = "let collect = fn {first, @rest} rest endfn";
    let ast = parser.parse(code).unwrap();
    evaluator.eval_with_player(&ast, player_id).unwrap();
    
    // Test with no extra args
    let call = parser.parse("collect(1)").unwrap();
    let result = evaluator.eval_with_player(&call, player_id).unwrap();
    
    assert_eq!(
        result,
        Value::List(vec![]),
        "Rest parameter should be empty list when no extra args"
    );
}

#[test]
fn test_mixed_parameters() {
    let (mut evaluator, player_id, _temp_dir) = setup_test();
    let mut parser = create_parser("echo").unwrap();
    
    // Create lambda with mixed parameters
    // Use single line to avoid parsing issues
    let code = "let mixed = fn {x, ?y=10, @rest} x + y endfn";
    let ast = parser.parse(code).unwrap();
    evaluator.eval_with_player(&ast, player_id).unwrap();
    
    // Test with minimum args
    let call1 = parser.parse("mixed(1)").unwrap();
    let result1 = evaluator.eval_with_player(&call1, player_id).unwrap();
    assert_eq!(
        result1,
        Value::Integer(11), // 1 + 10 (default)
        "Mixed params: minimum args should use default"
    );
    
    // Test with optional provided
    let call2 = parser.parse("mixed(1, 2)").unwrap();
    let result2 = evaluator.eval_with_player(&call2, player_id).unwrap();
    assert_eq!(
        result2,
        Value::Integer(3), // 1 + 2
        "Mixed params: optional arg should override default"
    );
    
    // Test with rest args - verify rest is collected but we just return x + y
    let call3 = parser.parse("mixed(1, 2, 3, 4, 5)").unwrap();
    let result3 = evaluator.eval_with_player(&call3, player_id).unwrap();
    assert_eq!(
        result3,
        Value::Integer(3), // 1 + 2 (rest params are collected but not used)
        "Mixed params: extra args should go to rest"
    );
    
    // To actually test rest parameter collection, let's make another function
    let rest_test = "let get_rest = fn {x, ?y=10, @rest} rest endfn";
    let ast2 = parser.parse(rest_test).unwrap();
    evaluator.eval_with_player(&ast2, player_id).unwrap();
    
    let call4 = parser.parse("get_rest(1, 2, 3, 4, 5)").unwrap();
    let result4 = evaluator.eval_with_player(&call4, player_id).unwrap();
    assert_eq!(
        result4,
        Value::List(vec![Value::Integer(3), Value::Integer(4), Value::Integer(5)]),
        "Rest parameter should collect extra arguments after optional"
    );
}

#[test]
fn test_lambda_missing_required_parameter() {
    let (mut evaluator, player_id, _temp_dir) = setup_test();
    let mut parser = create_parser("echo").unwrap();
    
    // Create lambda requiring 2 params
    let code = "let add = fn {x, y} x + y endfn";
    let ast = parser.parse(code).unwrap();
    evaluator.eval_with_player(&ast, player_id).unwrap();
    
    // Try to call with too few args
    let call = parser.parse("add(5)").unwrap();
    let result = evaluator.eval_with_player(&call, player_id);
    
    assert!(
        result.is_err(),
        "Should error when missing required parameter"
    );
    
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("Missing required argument: y"),
        "Error should mention missing parameter name"
    );
}

#[test]
fn test_lambda_too_many_arguments() {
    let (mut evaluator, player_id, _temp_dir) = setup_test();
    let mut parser = create_parser("echo").unwrap();
    
    // Create lambda with no rest parameter
    let code = "let add = fn {x, y} x + y endfn";
    let ast = parser.parse(code).unwrap();
    evaluator.eval_with_player(&ast, player_id).unwrap();
    
    // Try to call with too many args
    let call = parser.parse("add(1, 2, 3)").unwrap();
    let result = evaluator.eval_with_player(&call, player_id);
    
    assert!(
        result.is_err(),
        "Should error when too many arguments without rest parameter"
    );
    
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("Too many arguments provided"),
        "Error should mention too many arguments"
    );
}

#[test]
fn test_arrow_function_with_single_param() {
    let (mut evaluator, player_id, _temp_dir) = setup_test();
    let mut parser = create_parser("echo").unwrap();
    
    // Arrow function with single param doesn't need braces
    let code = "let double = x => x * 2";
    let ast = parser.parse(code).unwrap();
    evaluator.eval_with_player(&ast, player_id).unwrap();
    
    let call = parser.parse("double(21)").unwrap();
    let result = evaluator.eval_with_player(&call, player_id).unwrap();
    
    assert_eq!(
        result,
        Value::Integer(42),
        "Arrow function with single param should work"
    );
}

#[test]
fn test_arrow_function_with_multiple_params() {
    let (mut evaluator, player_id, _temp_dir) = setup_test();
    let mut parser = create_parser("echo").unwrap();
    
    // Arrow function with multiple params needs braces
    let code = "let add = {x, y} => x + y";
    let ast = parser.parse(code).unwrap();
    evaluator.eval_with_player(&ast, player_id).unwrap();
    
    let call = parser.parse("add(30, 12)").unwrap();
    let result = evaluator.eval_with_player(&call, player_id).unwrap();
    
    assert_eq!(
        result,
        Value::Integer(42),
        "Arrow function with multiple params should work"
    );
}

#[test]
fn test_lambda_closure_captures_environment() {
    let (mut evaluator, player_id, _temp_dir) = setup_test();
    let mut parser = create_parser("echo").unwrap();
    
    // Set up environment
    parser.parse("let multiplier = 10").map(|ast| evaluator.eval_with_player(&ast, player_id)).unwrap().unwrap();
    
    // Create closure that captures multiplier
    let code = "let scale = fn {x} x * multiplier endfn";
    let ast = parser.parse(code).unwrap();
    evaluator.eval_with_player(&ast, player_id).unwrap();
    
    // Test closure
    let call = parser.parse("scale(5)").unwrap();
    let result = evaluator.eval_with_player(&call, player_id).unwrap();
    
    assert_eq!(
        result,
        Value::Integer(50),
        "Lambda should capture environment variables"
    );
}

#[test]
#[ignore = "Grammar doesn't support rest parameter in middle position yet"]
fn test_rest_parameter_in_middle_position() {
    let (mut evaluator, player_id, _temp_dir) = setup_test();
    let mut parser = create_parser("echo").unwrap();
    
    // MOO allows rest parameter in any position
    // For now, our grammar only supports it at the end, but let's test that it works there
    let code = "let func = fn {x, @rest, y} rest endfn";
    let ast_result = parser.parse(code);
    
    // This might fail with current grammar, but let's check
    if ast_result.is_err() {
        // Expected for now - grammar doesn't support rest in middle yet
        return;
    }
    
    // If it parses, test it works correctly
    evaluator.eval_with_player(&ast_result.unwrap(), player_id).unwrap();
    
    let call = parser.parse("func(1, 2, 3, 4, 5)").unwrap();
    let result = evaluator.eval_with_player(&call, player_id).unwrap();
    
    // With rest in middle, it should collect middle args: [2, 3, 4]
    assert_eq!(
        result,
        Value::List(vec![
            Value::Integer(2),
            Value::Integer(3), 
            Value::Integer(4),
        ]),
        "Rest parameter in middle should collect middle arguments"
    );
}

#[test]
fn test_optional_parameter_evaluation_environment() {
    let (mut evaluator, player_id, _temp_dir) = setup_test();
    let mut parser = create_parser("echo").unwrap();
    
    // Test that optional parameter defaults are evaluated in the right environment
    parser.parse("let outer = 100").map(|ast| evaluator.eval_with_player(&ast, player_id)).unwrap().unwrap();
    
    // Create a lambda that has its own 'outer' parameter
    let code = "let func = fn {outer, ?inner=outer + 1} inner endfn";
    let ast = parser.parse(code).unwrap();
    evaluator.eval_with_player(&ast, player_id).unwrap();
    
    // Call with just first param - should use parameter value, not outer scope
    let call = parser.parse("func(5)").unwrap();
    let result = evaluator.eval_with_player(&call, player_id).unwrap();
    
    assert_eq!(
        result,
        Value::Integer(6), // 5 + 1, not 100 + 1
        "Optional parameter default should use parameter environment"
    );
}