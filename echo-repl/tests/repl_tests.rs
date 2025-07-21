use echo_repl::repl::{Repl, ReplCommand};
use tempfile::TempDir;

#[test]
fn test_repl_creation() {
    let temp_dir = TempDir::new().unwrap();
    let repl = Repl::with_storage_path(temp_dir.path()).unwrap();
    assert!(repl.is_running());
}

#[test]
fn test_repl_parse_command() {
    let temp_dir = TempDir::new().unwrap();
    let repl = Repl::with_storage_path(temp_dir.path()).unwrap();
    
    // Test basic command parsing
    assert_eq!(
        repl.parse_input(".help"),
        Ok(ReplCommand::Help)
    );
    
    assert_eq!(
        repl.parse_input(".quit"),
        Ok(ReplCommand::Quit)
    );
    
    // Test Echo code parsing
    assert_eq!(
        repl.parse_input("let x = 42;"),
        Ok(ReplCommand::Execute("let x = 42;".to_string()))
    );
}

#[test]
fn test_repl_execute_simple_expression() {
    let temp_dir = TempDir::new().unwrap();
    let mut repl = Repl::with_storage_path(temp_dir.path()).unwrap();
    
    // Test simple arithmetic
    let result = repl.execute("2 + 2");
    match result {
        Ok((value, _duration)) => assert_eq!(value, "4"),
        Err(e) => panic!("Execution failed: {:?}", e),
    }
}

#[test]
fn test_repl_variable_binding() {
    let temp_dir = TempDir::new().unwrap();
    let mut repl = Repl::with_storage_path(temp_dir.path()).unwrap();
    
    // Test variable binding
    repl.execute("let x = 42;").unwrap();
    let result = repl.execute("x");
    match result {
        Ok((value, _duration)) => assert_eq!(value, "42"),
        Err(e) => panic!("Variable lookup failed: {:?}", e),
    }
}

#[test]
fn test_repl_object_creation() {
    let temp_dir = TempDir::new().unwrap();
    let mut repl = Repl::with_storage_path(temp_dir.path()).unwrap();
    
    // Test simple object creation
    let code = r#"
        object simple_object
            property name = "test";
        endobject
    "#;
    
    let result = repl.execute(code);
    match &result {
        Ok((msg, _duration)) => assert!(msg.contains("object created")),
        Err(e) => panic!("Object creation failed: {:?}", e),
    }
}