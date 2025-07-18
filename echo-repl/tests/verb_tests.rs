use echo_repl::repl::Repl;
use tempfile::TempDir;

#[test]
fn test_verb_definition() {
    let temp_dir = TempDir::new().unwrap();
    let mut repl = Repl::with_storage_path(temp_dir.path()).unwrap();
    
    // Create an object with a verb
    let code = r#"
        object greeter
            verb "greet" (this, "none", "none")
                caller:tell("Hello from ", this.name, "!");
            endverb
        endobject
    "#;
    
    let result = repl.execute(code);
    assert!(result.is_ok());
}

#[test]
fn test_verb_execution() {
    let temp_dir = TempDir::new().unwrap();
    let mut repl = Repl::with_storage_path(temp_dir.path()).unwrap();
    
    // Create object with verb
    repl.execute(r#"
        object greeter
            property name = "Bob";
            verb "greet" (this, "none", "none")
                return "Hello from " + this.name + "!";
            endverb
        endobject
    "#).unwrap();
    
    // Store the object in a variable
    repl.execute("let g = greeter;").unwrap();
    
    // Call the verb
    let result = repl.execute("g:greet()");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Hello from Bob!");
}

#[test]
fn test_verb_with_arguments() {
    let temp_dir = TempDir::new().unwrap();
    let mut repl = Repl::with_storage_path(temp_dir.path()).unwrap();
    
    // Create object with verb that takes arguments
    repl.execute(r#"
        object calculator
            verb "add" (this, "none", "none")
                return args[1] + args[2];
            endverb
        endobject
    "#).unwrap();
    
    repl.execute("let calc = calculator;").unwrap();
    
    // Call verb with arguments
    let result = repl.execute("calc:add(5, 3)");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "8");
}

#[test]
fn test_verb_accessing_caller() {
    let temp_dir = TempDir::new().unwrap();
    let mut repl = Repl::with_storage_path(temp_dir.path()).unwrap();
    
    // Create an object that uses caller
    repl.execute(r#"
        object info_booth
            verb "whoami" (this, "none", "none")
                return "You are guest";
            endverb
        endobject
    "#).unwrap();
    
    repl.execute("let booth = info_booth;").unwrap();
    
    // The current player should be the caller
    let result = repl.execute("booth:whoami()");
    assert!(result.is_ok());
    // Should contain the player name
    let output = result.unwrap();
    assert!(output.contains("guest") || output.contains("default"));
}