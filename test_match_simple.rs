use echo_core::parser::echo::grammar::parse_echo;

fn main() {
    // Test 1: Simple match without guard
    let input1 = "match 42 case 10 => \"ten\" endmatch";
    match parse_echo(input1) {
        Ok(_) => println!("Test 1 passed"),
        Err(e) => println!("Test 1 failed: {:?}", e),
    }
    
    // Test 2: Match with identifier pattern (no guard)
    let input2 = "match 42 case x => \"any\" endmatch";
    match parse_echo(input2) {
        Ok(_) => println!("Test 2 passed"),
        Err(e) => println!("Test 2 failed: {:?}", e),
    }
    
    // Test 3: Match with guard
    let input3 = "match 42 case x when x > 10 => \"big\" endmatch";
    match parse_echo(input3) {
        Ok(_) => println!("Test 3 passed"),
        Err(e) => println!("Test 3 failed: {:?}", e),
    }
}