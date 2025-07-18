use echo_repl::repl::{Repl, ReplCommand};

#[test]
fn test_repl_creation() {
    let repl = Repl::new();
    assert!(repl.is_running());
}

#[test]
fn test_repl_parse_command() {
    let repl = Repl::new();
    
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
    let mut repl = Repl::new();
    
    // Test simple arithmetic
    let result = repl.execute("2 + 2");
    assert_eq!(result, Ok("4".to_string()));
}

#[test]
fn test_repl_variable_binding() {
    let mut repl = Repl::new();
    
    // Test variable binding
    repl.execute("let x = 42;").unwrap();
    let result = repl.execute("x");
    assert_eq!(result, Ok("42".to_string()));
}

#[test]
fn test_repl_object_creation() {
    let mut repl = Repl::new();
    
    // Test simple object creation
    let code = r#"
        object simple_object
            property name = "test";
        endobject
    "#;
    
    let result = repl.execute(code);
    assert!(result.is_ok());
    assert!(result.unwrap().contains("object created"));
}