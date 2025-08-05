use echo_core::parser::echo::grammar::parse_echo;

fn main() {
    let input = r#"
match 42
case x when x > 10 => "big"
case _ => "small"
endmatch
"#;
    
    match parse_echo(input) {
        Ok(ast) => println!("Parsed successfully: {:#?}", ast),
        Err(e) => println!("Parse failed: {:?}", e),
    }
}