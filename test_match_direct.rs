use echo_core::parser::echo::grammar::parse_echo;

fn main() {
    let input = r#"
match 42
case 10 => "ten"
case 42 => "forty-two"  
case _ => "other"
endmatch
"#;
    
    match parse_echo(input) {
        Ok(ast) => println!("Parsed successfully: {:#?}", ast),
        Err(e) => println!("Parse failed: {:?}", e),
    }
}