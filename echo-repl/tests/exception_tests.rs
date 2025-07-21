use std::sync::Arc;
use echo_repl::evaluator::{create_evaluator, Value};
use echo_repl::parser::create_parser;
use echo_repl::storage::Storage;
use tempfile::tempdir;

// Helper function to parse Echo code
fn parse_echo(code: &str) -> anyhow::Result<echo_repl::ast::EchoAst> {
    let mut parser = create_parser("echo")?;
    parser.parse(code)
}

#[test]
fn test_simple_throw_catch() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let storage = Arc::new(Storage::new(temp_dir.path()).expect("Failed to create storage"));
    let mut evaluator = create_evaluator(storage).unwrap();
    
    // Create a player
    let player_id = evaluator.create_player("test_player").unwrap();
    evaluator.switch_player(player_id).unwrap();
    
    // Test: try throw "error"; catch (e) return e; endtry
    let code = "try throw \"error\"; catch (e) return e; endtry";
    let ast = parse_echo(code).expect("Should parse successfully");
    
    let result = evaluator.eval(&ast).expect("Should evaluate successfully");
    assert_eq!(result, Value::String("error".to_string()));
}

#[test]
fn test_throw_catch_no_exception_var() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let storage = Arc::new(Storage::new(temp_dir.path()).expect("Failed to create storage"));
    let mut evaluator = create_evaluator(storage).unwrap();
    
    // Create a player
    let player_id = evaluator.create_player("test_player").unwrap();
    evaluator.switch_player(player_id).unwrap();
    
    // Test: try throw 42; catch () return "caught"; endtry
    let code = "try throw 42; catch () return \"caught\"; endtry";
    let ast = parse_echo(code).expect("Should parse successfully");
    
    let result = evaluator.eval(&ast).expect("Should evaluate successfully");
    assert_eq!(result, Value::String("caught".to_string()));
}

#[test]
fn test_no_exception_thrown() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let storage = Arc::new(Storage::new(temp_dir.path()).expect("Failed to create storage"));
    let mut evaluator = create_evaluator(storage).unwrap();
    
    // Create a player
    let player_id = evaluator.create_player("test_player").unwrap();
    evaluator.switch_player(player_id).unwrap();
    
    // Test: try return "success"; catch (e) return "error"; endtry
    let code = "try return \"success\"; catch (e) return \"error\"; endtry";
    let ast = parse_echo(code).expect("Should parse successfully");
    
    let result = evaluator.eval(&ast).expect("Should evaluate successfully");
    assert_eq!(result, Value::String("success".to_string()));
}

#[test]
fn test_uncaught_exception() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let storage = Arc::new(Storage::new(temp_dir.path()).expect("Failed to create storage"));
    let mut evaluator = create_evaluator(storage).unwrap();
    
    // Create a player
    let player_id = evaluator.create_player("test_player").unwrap();
    evaluator.switch_player(player_id).unwrap();
    
    // Test: throw "uncaught error";
    let code = "throw \"uncaught error\";";
    let ast = parse_echo(code).expect("Should parse successfully");
    
    let result = evaluator.eval(&ast);
    assert!(result.is_err());
    let error_msg = format!("{}", result.unwrap_err());
    assert!(error_msg.contains("uncaught error"));
}

#[test]
fn test_catch_with_conditional() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let storage = Arc::new(Storage::new(temp_dir.path()).expect("Failed to create storage"));
    let mut evaluator = create_evaluator(storage).unwrap();
    
    // Create a player
    let player_id = evaluator.create_player("test_player").unwrap();
    evaluator.switch_player(player_id).unwrap();
    
    // Test: try throw 1; catch (e) if (e == 1) return "one"; endif endtry
    let code = "try throw 1; catch (e) if (e == 1) return \"one\"; endif endtry";
    let ast = parse_echo(code).expect("Should parse successfully");
    
    let result = evaluator.eval(&ast).expect("Should evaluate successfully");
    assert_eq!(result, Value::String("one".to_string()));
}

#[test]
fn test_nested_try_catch() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let storage = Arc::new(Storage::new(temp_dir.path()).expect("Failed to create storage"));
    let mut evaluator = create_evaluator(storage).unwrap();
    
    // Create a player
    let player_id = evaluator.create_player("test_player").unwrap();
    evaluator.switch_player(player_id).unwrap();
    
    // Test: try try throw "inner"; catch (e) throw "outer"; endtry catch (e) return e; endtry
    let code = "try try throw \"inner\"; catch (e) throw \"outer\"; endtry catch (e) return e; endtry";
    let ast = parse_echo(code).expect("Should parse successfully");
    
    let result = evaluator.eval(&ast).expect("Should evaluate successfully");
    assert_eq!(result, Value::String("outer".to_string()));
}

#[test]
fn test_exception_in_loop() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let storage = Arc::new(Storage::new(temp_dir.path()).expect("Failed to create storage"));
    let mut evaluator = create_evaluator(storage).unwrap();
    
    // Create a player
    let player_id = evaluator.create_player("test_player").unwrap();
    evaluator.switch_player(player_id).unwrap();
    
    // Test: try for (i in [1, 2, 3]) if (i == 2) throw "error"; endif endfor catch (e) return e; endtry
    let code = "try for (i in [1, 2, 3]) if (i == 2) throw \"error\"; endif endfor catch (e) return e; endtry";
    let ast = parse_echo(code).expect("Should parse successfully");
    
    let result = evaluator.eval(&ast).expect("Should evaluate successfully");
    assert_eq!(result, Value::String("error".to_string()));
}

#[test]
fn test_exception_with_different_types() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let storage = Arc::new(Storage::new(temp_dir.path()).expect("Failed to create storage"));
    let mut evaluator = create_evaluator(storage).unwrap();
    
    // Create a player
    let player_id = evaluator.create_player("test_player").unwrap();
    evaluator.switch_player(player_id).unwrap();
    
    // Test throwing different types
    let test_cases = vec![
        ("try throw 42; catch (e) return e; endtry", Value::Integer(42)),
        ("try throw \"text\"; catch (e) return e; endtry", Value::String("text".to_string())),
        ("try throw true; catch (e) return e; endtry", Value::Boolean(true)),
        ("try throw [1, 2, 3]; catch (e) return e; endtry", Value::List(vec![Value::Integer(1), Value::Integer(2), Value::Integer(3)])),
    ];
    
    for (code, expected) in test_cases {
        let ast = parse_echo(code).expect("Should parse successfully");
        let result = evaluator.eval(&ast).expect("Should evaluate successfully");
        assert_eq!(result, expected);
    }
}

#[test]
fn test_catch_outside_try() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let storage = Arc::new(Storage::new(temp_dir.path()).expect("Failed to create storage"));
    let mut evaluator = create_evaluator(storage).unwrap();
    
    // Create a player
    let player_id = evaluator.create_player("test_player").unwrap();
    evaluator.switch_player(player_id).unwrap();
    
    // Test: catch (e) return e; (should fail at parse time)
    let code = "catch (e) return e;";
    let result = parse_echo(code);
    assert!(result.is_err(), "Catch outside try should fail to parse");
}

#[test]
fn test_exception_variable_scope() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let storage = Arc::new(Storage::new(temp_dir.path()).expect("Failed to create storage"));
    let mut evaluator = create_evaluator(storage).unwrap();
    
    // Create a player
    let player_id = evaluator.create_player("test_player").unwrap();
    evaluator.switch_player(player_id).unwrap();
    
    // Set up a variable before try block
    let setup_code = "let e = \"original\";";
    let setup_ast = parse_echo(setup_code).expect("Should parse successfully");
    evaluator.eval(&setup_ast).expect("Should evaluate successfully");
    
    // Test: try throw "exception"; catch (e) let inner = e; endtry return e;
    let code = "try throw \"exception\"; catch (e) let inner = e; endtry";
    let ast = parse_echo(code).expect("Should parse successfully");
    evaluator.eval(&ast).expect("Should evaluate successfully");
    
    // Check that the exception variable was set in the catch block
    let check_code = "inner";
    let check_ast = parse_echo(check_code).expect("Should parse successfully");
    let result = evaluator.eval(&check_ast).expect("Should evaluate successfully");
    assert_eq!(result, Value::String("exception".to_string()));
    
    // Check that the original variable is still accessible
    let original_code = "e";
    let original_ast = parse_echo(original_code).expect("Should parse successfully");
    let original_result = evaluator.eval(&original_ast).expect("Should evaluate successfully");
    assert_eq!(original_result, Value::String("exception".to_string())); // Exception variable overwrote original
}