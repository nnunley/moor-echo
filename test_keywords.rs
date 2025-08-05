use echo_core::parser::echo::grammar::parse_echo;

fn main() {
    let keywords = vec!["if", "while", "for", "match", "case", "when", "let", "const"];
    
    for kw in keywords {
        print!("{}: ", kw);
        match parse_echo(kw) {
            Ok(ast) => println!("{:?}", ast),
            Err(e) => println!("Error: {:?}", e),
        }
    }
}