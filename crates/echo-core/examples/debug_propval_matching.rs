use pest::Parser;
use echo_core::parser::lambdamoo_db_parser::{LambdaMooDbParser, Rule};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing what propval rule actually matches...");
    
    // Test 1: What happens when propval encounters object header?
    println!("=== Test 1: propval on object header content ===");
    let object_header_line = "#1";
    
    match LambdaMooDbParser::parse(Rule::propval, object_header_line) {
        Ok(mut pairs) => {
            if let Some(pair) = pairs.next() {
                println!("🚨 propval matched object header: '{}'", pair.as_str());
            }
        }
        Err(_) => println!("✅ propval correctly rejected object header"),
    }
    
    // Test 2: What about multiline object content?
    println!("\n=== Test 2: propval on multiline object content ===");
    let multiline_content = "#1\nRoot Class\n\n16";
    
    match LambdaMooDbParser::parse(Rule::propval, multiline_content) {
        Ok(mut pairs) => {
            if let Some(pair) = pairs.next() {
                println!("🚨 propval matched multiline content: '{}'", pair.as_str().replace('\n', "\\n"));
            }
        }
        Err(_) => println!("✅ propval correctly rejected multiline content"),
    }
    
    // Test 3: What does propval expect? Let's see the structure
    // propval = { value ~ prop_owner ~ newline ~ prop_perms ~ newline }
    println!("\n=== Test 3: Testing valid propval structure ===");
    let valid_propval = "1\nSome string\n#0\n5";
    
    match LambdaMooDbParser::parse(Rule::propval, valid_propval) {
        Ok(mut pairs) => {
            if let Some(pair) = pairs.next() {
                println!("✅ Valid propval parsed: '{}'", pair.as_str().replace('\n', "\\n"));
                
                // Analyze the internal structure
                for inner in pair.into_inner() {
                    match inner.as_rule() {
                        Rule::value => println!("  value: '{}'", inner.as_str().replace('\n', "\\n")),
                        Rule::prop_owner => println!("  prop_owner: '{}'", inner.as_str()),
                        Rule::prop_perms => println!("  prop_perms: '{}'", inner.as_str()),
                        _ => println!("  other: {:?} = '{}'", inner.as_rule(), inner.as_str()),
                    }
                }
            }
        }
        Err(e) => println!("❌ Valid propval failed: {}", e),
    }
    
    // Test 4: The critical test - what if value consumes too much?
    println!("\n=== Test 4: Does value consume object header? ===");
    let tricky_value_content = "#1\nRoot Class\n\n16\n#0\n5";
    
    match LambdaMooDbParser::parse(Rule::propval, tricky_value_content) {
        Ok(mut pairs) => {
            if let Some(pair) = pairs.next() {
                println!("🚨 CRITICAL: propval consumed: '{}'", pair.as_str().replace('\n', "\\n"));
                println!("This means value rule is consuming object headers!");
            }
        }
        Err(e) => println!("✅ propval correctly failed: {}", e),
    }
    
    Ok(())
}