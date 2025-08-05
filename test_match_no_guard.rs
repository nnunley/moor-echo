use echo_core::parser::echo::grammar::parse_echo;

fn main() {
    let input = r#"
match 42
case x => "any"
endmatch
"#;
    
    match parse_echo(input) {
        Ok(ast) => println!("Parsed successfully"),
        Err(e) => println!("Parse failed: {:?}", e),
    }
}