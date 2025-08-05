fn main() -> Result<(), Box<dyn std::error::Error>> {
    let test_code = r#"
object #1
  name: "Test Object"
endobject
"#;

    let mut parser = echo_core::parser::create_parser("moo")?;
    match parser.parse(test_code) {
        Ok(ast) => println!("Success: {:?}", ast),
        Err(e) => println!("Error: {}", e),
    }
    
    Ok(())
}