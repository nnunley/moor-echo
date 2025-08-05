use echo_core::parser::echo::grammar::parse_echo;

fn main() {
    // Test if "when" is being recognized as a keyword
    let tests = vec![
        ("when", "bare when keyword"),
        ("x when", "identifier then when"),
        ("case x => y", "case without when"),
        ("case x when", "case with when but no condition"),
    ];
    
    for (input, desc) in tests {
        println!("\nTesting: {} ('{}')", desc, input);
        match parse_echo(input) {
            Ok(ast) => println!("  Parsed as: {:?}", ast),
            Err(e) => println!("  Failed: {:?}", e),
        }
    }
}