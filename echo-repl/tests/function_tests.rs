use std::sync::Arc;
use echo_repl::evaluator::{create_evaluator, Value};
use echo_repl::parser::grammar::parse_echo;
use echo_repl::storage::Storage;
use tempfile::tempdir;

#[test]
fn test_simple_function_definition() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let storage = Arc::new(Storage::new(temp_dir.path()).expect("Failed to create storage"));
    let mut evaluator = create_evaluator(storage).unwrap();
    
    // Create a player
    let player_id = evaluator.create_player("test_player").unwrap();
    evaluator.switch_player(player_id).unwrap();
    
    // Test: let add = fn(a, b) return a + b; endfn
    let code = "let add = fn(a, b) return a + b; endfn";
    let ast = parse_echo(code).expect("Should parse successfully");
    
    let result = evaluator.eval(&ast).expect("Should evaluate successfully");
    
    // Function definitions should return the function value
    match result {
        Value::Function(func) => {
            assert_eq!(func.name, "add");
            assert_eq!(func.parameters, vec!["a", "b"]);
            assert_eq!(func.body.len(), 1);
        }
        _ => panic!("Expected Function value"),
    }
}

#[test]
fn test_function_call() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let storage = Arc::new(Storage::new(temp_dir.path()).expect("Failed to create storage"));
    let mut evaluator = create_evaluator(storage).unwrap();
    
    // Create a player
    let player_id = evaluator.create_player("test_player").unwrap();
    evaluator.switch_player(player_id).unwrap();
    
    // Define function
    let def_code = "let add = fn(a, b) return a + b; endfn";
    let def_ast = parse_echo(def_code).expect("Should parse successfully");
    evaluator.eval(&def_ast).expect("Should evaluate successfully");
    
    // Call function
    let call_code = "add(5, 3)";
    let call_ast = parse_echo(call_code).expect("Should parse successfully");
    let result = evaluator.eval(&call_ast).expect("Should evaluate successfully");
    
    assert_eq!(result, Value::Integer(8));
}

#[test]
fn test_function_with_no_parameters() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let storage = Arc::new(Storage::new(temp_dir.path()).expect("Failed to create storage"));
    let mut evaluator = create_evaluator(storage).unwrap();
    
    // Create a player
    let player_id = evaluator.create_player("test_player").unwrap();
    evaluator.switch_player(player_id).unwrap();
    
    // Define function
    let def_code = "let get_answer = fn() return 42; endfn";
    let def_ast = parse_echo(def_code).expect("Should parse successfully");
    evaluator.eval(&def_ast).expect("Should evaluate successfully");
    
    // Call function
    let call_code = "get_answer()";
    let call_ast = parse_echo(call_code).expect("Should parse successfully");
    let result = evaluator.eval(&call_ast).expect("Should evaluate successfully");
    
    assert_eq!(result, Value::Integer(42));
}

#[test]
fn test_function_with_local_variables() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let storage = Arc::new(Storage::new(temp_dir.path()).expect("Failed to create storage"));
    let mut evaluator = create_evaluator(storage).unwrap();
    
    // Create a player
    let player_id = evaluator.create_player("test_player").unwrap();
    evaluator.switch_player(player_id).unwrap();
    
    // Define function that uses local variables
    let def_code = "let calculate = fn(x) let doubled = x + x; return doubled + 1; endfn";
    let def_ast = parse_echo(def_code).expect("Should parse successfully");
    evaluator.eval(&def_ast).expect("Should evaluate successfully");
    
    // Call function
    let call_code = "calculate(5)";
    let call_ast = parse_echo(call_code).expect("Should parse successfully");
    let result = evaluator.eval(&call_ast).expect("Should evaluate successfully");
    
    assert_eq!(result, Value::Integer(11)); // 5 + 5 + 1
}

#[test]
fn test_function_parameter_shadowing() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let storage = Arc::new(Storage::new(temp_dir.path()).expect("Failed to create storage"));
    let mut evaluator = create_evaluator(storage).unwrap();
    
    // Create a player
    let player_id = evaluator.create_player("test_player").unwrap();
    evaluator.switch_player(player_id).unwrap();
    
    // Set global variable
    let global_code = "let x = 100;";
    let global_ast = parse_echo(global_code).expect("Should parse successfully");
    evaluator.eval(&global_ast).expect("Should evaluate successfully");
    
    // Define function with parameter that shadows global
    let def_code = "let test_shadow = fn(x) return x + 1; endfn";
    let def_ast = parse_echo(def_code).expect("Should parse successfully");
    evaluator.eval(&def_ast).expect("Should evaluate successfully");
    
    // Call function - should use parameter, not global
    let call_code = "test_shadow(5)";
    let call_ast = parse_echo(call_code).expect("Should parse successfully");
    let result = evaluator.eval(&call_ast).expect("Should evaluate successfully");
    
    assert_eq!(result, Value::Integer(6)); // Should use parameter x=5, not global x=100
}

#[test]
fn test_function_wrong_argument_count() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let storage = Arc::new(Storage::new(temp_dir.path()).expect("Failed to create storage"));
    let mut evaluator = create_evaluator(storage).unwrap();
    
    // Create a player
    let player_id = evaluator.create_player("test_player").unwrap();
    evaluator.switch_player(player_id).unwrap();
    
    // Define function
    let def_code = "let add = fn(a, b) return a + b; endfn";
    let def_ast = parse_echo(def_code).expect("Should parse successfully");
    evaluator.eval(&def_ast).expect("Should evaluate successfully");
    
    // Call function with wrong number of arguments
    let call_code = "add(5)";
    let call_ast = parse_echo(call_code).expect("Should parse successfully");
    let result = evaluator.eval(&call_ast);
    
    assert!(result.is_err());
    let error_msg = format!("{}", result.unwrap_err());
    assert!(error_msg.contains("expects 2 arguments, got 1"));
}

#[test]
fn test_undefined_function() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let storage = Arc::new(Storage::new(temp_dir.path()).expect("Failed to create storage"));
    let mut evaluator = create_evaluator(storage).unwrap();
    
    // Create a player
    let player_id = evaluator.create_player("test_player").unwrap();
    evaluator.switch_player(player_id).unwrap();
    
    // Call undefined function
    let call_code = "undefined_function()";
    let call_ast = parse_echo(call_code).expect("Should parse successfully");
    let result = evaluator.eval(&call_ast);
    
    assert!(result.is_err());
    let error_msg = format!("{}", result.unwrap_err());
    assert!(error_msg.contains("Function 'undefined_function' not found"));
}

#[test]
fn test_recursive_function() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let storage = Arc::new(Storage::new(temp_dir.path()).expect("Failed to create storage"));
    let mut evaluator = create_evaluator(storage).unwrap();
    
    // Create a player
    let player_id = evaluator.create_player("test_player").unwrap();
    evaluator.switch_player(player_id).unwrap();
    
    // Define recursive factorial function
    let def_code = "let factorial = fn(n) if (n <= 1) return 1; endif return n * factorial(n - 1); endfn";
    let def_ast = parse_echo(def_code).expect("Should parse successfully");
    evaluator.eval(&def_ast).expect("Should evaluate successfully");
    
    // Call recursive function
    let call_code = "factorial(5)";
    let call_ast = parse_echo(call_code).expect("Should parse successfully");
    let result = evaluator.eval(&call_ast).expect("Should evaluate successfully");
    
    assert_eq!(result, Value::Integer(120)); // 5! = 120
}

#[test]
fn test_function_with_string_parameters() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let storage = Arc::new(Storage::new(temp_dir.path()).expect("Failed to create storage"));
    let mut evaluator = create_evaluator(storage).unwrap();
    
    // Create a player
    let player_id = evaluator.create_player("test_player").unwrap();
    evaluator.switch_player(player_id).unwrap();
    
    // Define function that works with strings
    let def_code = "let greet = fn(name) return \"Hello, \" + name + \"!\"; endfn";
    let def_ast = parse_echo(def_code).expect("Should parse successfully");
    evaluator.eval(&def_ast).expect("Should evaluate successfully");
    
    // Call function with string argument
    let call_code = "greet(\"Alice\")";
    let call_ast = parse_echo(call_code).expect("Should parse successfully");
    let result = evaluator.eval(&call_ast).expect("Should evaluate successfully");
    
    assert_eq!(result, Value::String("Hello, Alice!".to_string()));
}

#[test]
fn test_function_with_multiple_statements() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let storage = Arc::new(Storage::new(temp_dir.path()).expect("Failed to create storage"));
    let mut evaluator = create_evaluator(storage).unwrap();
    
    // Create a player
    let player_id = evaluator.create_player("test_player").unwrap();
    evaluator.switch_player(player_id).unwrap();
    
    // Define function with multiple statements
    let def_code = "let complex = fn(x, y) let sum = x + y; let product = x * y; return sum + product; endfn";
    let def_ast = parse_echo(def_code).expect("Should parse successfully");
    evaluator.eval(&def_ast).expect("Should evaluate successfully");
    
    // Call function
    let call_code = "complex(3, 4)";
    let call_ast = parse_echo(call_code).expect("Should parse successfully");
    let result = evaluator.eval(&call_ast).expect("Should evaluate successfully");
    
    assert_eq!(result, Value::Integer(19)); // (3 + 4) + (3 * 4) = 7 + 12 = 19
}

#[test]
fn test_function_calling_function() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let storage = Arc::new(Storage::new(temp_dir.path()).expect("Failed to create storage"));
    let mut evaluator = create_evaluator(storage).unwrap();
    
    // Create a player
    let player_id = evaluator.create_player("test_player").unwrap();
    evaluator.switch_player(player_id).unwrap();
    
    // Define helper function
    let helper_code = "let double = fn(x) return x * 2; endfn";
    let helper_ast = parse_echo(helper_code).expect("Should parse successfully");
    evaluator.eval(&helper_ast).expect("Should evaluate successfully");
    
    // Define main function that calls helper
    let main_code = "let quadruple = fn(x) return double(double(x)); endfn";
    let main_ast = parse_echo(main_code).expect("Should parse successfully");
    evaluator.eval(&main_ast).expect("Should evaluate successfully");
    
    // Call main function
    let call_code = "quadruple(5)";
    let call_ast = parse_echo(call_code).expect("Should parse successfully");
    let result = evaluator.eval(&call_ast).expect("Should evaluate successfully");
    
    assert_eq!(result, Value::Integer(20)); // double(double(5)) = double(10) = 20
}