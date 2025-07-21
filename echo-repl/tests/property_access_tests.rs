use echo_repl::repl::Repl;
use tempfile::TempDir;

#[test]
fn test_property_access_simple() {
    let temp_dir = TempDir::new().unwrap();
    let mut repl = Repl::with_storage_path(temp_dir.path()).unwrap();
    
    // Create object with property
    repl.execute(r#"
        object person
            property name = "Alice";
            property age = 30;
        endobject
    "#).unwrap();
    
    // Store the object in a variable
    repl.execute("let p = person;").unwrap();
    
    // Access properties using dot notation
    let result = repl.execute("p.name");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().0, "Alice");
    
    let result = repl.execute("p.age");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().0, "30");
}

#[test]
fn test_property_access_in_expression() {
    let temp_dir = TempDir::new().unwrap();
    let mut repl = Repl::with_storage_path(temp_dir.path()).unwrap();
    
    // Create object with numeric property
    repl.execute(r#"
        object counter
            property count = 5;
        endobject
    "#).unwrap();
    
    repl.execute("let c = counter;").unwrap();
    
    // First test just the property access
    let result = repl.execute("c.count");
    if result.is_err() {
        eprintln!("Error accessing property: {:?}", result.as_ref().unwrap_err());
    }
    assert!(result.is_ok());
    assert_eq!(result.unwrap().0, "5");
    
    // Use property in arithmetic expression
    let result = repl.execute("c.count + 10");
    if result.is_err() {
        eprintln!("Error in expression: {:?}", result.as_ref().unwrap_err());
    }
    assert!(result.is_ok());
    assert_eq!(result.unwrap().0, "15");
}

#[test]
fn test_property_access_in_verb() {
    let temp_dir = TempDir::new().unwrap();
    let mut repl = Repl::with_storage_path(temp_dir.path()).unwrap();
    
    // Create object with property and verb that accesses it
    repl.execute(r#"
        object greeter
            property greeting = "Hello";
            property name = "World";
            verb "greet" (this, "none", "none")
                return this.greeting + " " + this.name + "!";
            endverb
        endobject
    "#).unwrap();
    
    repl.execute("let g = greeter;").unwrap();
    
    // Call verb that uses property access
    let result = repl.execute("g:greet()");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().0, "Hello World!");
}

#[test]
fn test_property_access_nonexistent() {
    let temp_dir = TempDir::new().unwrap();
    let mut repl = Repl::with_storage_path(temp_dir.path()).unwrap();
    
    // Create object with property
    repl.execute(r#"
        object person
            property name = "Bob";
        endobject
    "#).unwrap();
    
    repl.execute("let p = person;").unwrap();
    
    // Try to access nonexistent property
    let result = repl.execute("p.nonexistent");
    assert!(result.is_err());
}