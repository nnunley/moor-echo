use echo_repl::parser::create_parser;

fn main() -> anyhow::Result<()> {
    let mut parser = create_parser("echo")?;
    
    let test_cases = vec![
        "object foo
    verb greet {}
        return \"Hello!\"
    endverb
endobject",
        "object foo
    property name = \"Test\"
    verb greet {name}
        return \"Hello, \" + name
    endverb
endobject",
    ];
    
    for test_case in test_cases {
        println!("\n=== Testing: ===\n{}", test_case);
        println!("================");
        match parser.parse(test_case) {
            Ok(ast) => println!("Success: {:#?}", ast),
            Err(e) => println!("Error: {}", e),
        }
    }
    
    Ok(())
}