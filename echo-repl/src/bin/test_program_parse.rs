use echo_repl::parser::create_parser;
use echo_repl::parser::Parser;

fn main() {
    let program = "let mul = fn {x, y}
  x * y  
endfn
mul(3, 4)";

    let mut parser = create_parser("echo").expect("Failed to create parser");
    
    println!("=== Testing parse_program ===");
    match parser.parse_program(program) {
        Ok(ast) => println!("Success: {:#?}", ast),
        Err(e) => println!("Error: {}", e),
    }
    
    println!("\n=== Testing parse (single statement) ===");
    match parser.parse("let mul = fn {x, y}\n  x * y\nendfn") {
        Ok(ast) => println!("Success: {:#?}", ast),
        Err(e) => println!("Error: {}", e),
    }
}