use echo_repl::parser::create_parser;

fn main() -> anyhow::Result<()> {
    let mut parser = create_parser("echo")?;
    
    let test_cases = vec![
        "object hello",
        "object hello\n  property greeting = \"Hello\"\nendobject",
        "object hello\nproperty greeting = \"Hello\"\nendobject",
        // While loop tests
        "while (i < 3) i = i + 1 endwhile",
        "while (i < 3)\n  i = i + 1\nendwhile",
        // Program tests
        "let i = 0\nwhile (i < 3)\n  i = i + 1\nendwhile",
    ];
    
    for test_case in test_cases {
        println!("\n=== Testing: {:?} ===", test_case);
        match parser.parse(test_case) {
            Ok(ast) => println!("Success: {:?}", ast),
            Err(e) => println!("Error: {}", e),
        }
    }
    
    Ok(())
}