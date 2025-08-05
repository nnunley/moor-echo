use echo_core::parser::echo::grammar::{parse_echo, GuardClause, MatchArm, MatchPattern};

fn main() {
    // Let's try to understand if GuardClause can be parsed at all
    
    // Test a complete match expression but simpler
    let input = r#"
match 1
case 1 => "one"
endmatch
"#;
    
    match parse_echo(input) {
        Ok(ast) => println!("Simple match parsed OK"),
        Err(e) => println!("Simple match failed: {:?}", e),
    }
}