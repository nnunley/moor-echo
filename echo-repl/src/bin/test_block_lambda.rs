use echo_repl::parser::create_parser;

fn main() {
    let test_cases = vec![
        // Simple block function
        "fn {} 42 endfn",
        "fn {x} x endfn", 
        "fn {x, y} x + y endfn",
        
        // With assignment
        "let f = fn {} 42 endfn",
        "let add = fn {x, y} x + y endfn",
        
        // Multi-line
        "fn {x}\n  x * 2\nendfn",
        "let f = fn {x, y}\n  let sum = x + y\n  sum * 2\nendfn",
    ];

    let mut parser = create_parser("echo").expect("Failed to create parser");
    
    for test in test_cases {
        println!("\n=== Testing: {:?} ===", test);
        match parser.parse(test) {
            Ok(ast) => println!("Success: {:?}", ast),
            Err(e) => println!("Error: {}", e),
        }
    }
}