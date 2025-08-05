use pest::Parser;
use echo_core::parser::lambdamoo_db_parser::{LambdaMooDbParser, Rule};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing verb->property transition issue...");
    
    // Test 1: Verb definitions with exactly the structure from the database
    println!("=== Test 1: Full verb section as in database ===");
    let verb_section = r#"2
do_start_script
3
173
-1
do_login_command
3
173
-1"#;
    
    match LambdaMooDbParser::parse(Rule::verb_definitions, verb_section) {
        Ok(pairs) => {
            println!("✅ verb_definitions parsed");
            for pair in pairs {
                println!("  Full match: '{}'", pair.as_str());
                let full_match = pair.as_str();
                println!("  Length: {}", full_match.len());
                println!("  Last 10 chars: {:?}", full_match.chars().rev().take(10).collect::<Vec<_>>());
            }
        }
        Err(e) => println!("❌ verb_definitions failed: {}", e),
    }
    
    // Test 2: Add the property count immediately after
    println!("\n=== Test 2: Verb section + property count ===");
    let verb_plus_prop = r#"2
do_start_script
3
173
-1
do_login_command
3
173
-1
0"#;
    
    // This should NOT parse as verb_definitions because there's extra content
    match LambdaMooDbParser::parse(Rule::verb_definitions, verb_plus_prop) {
        Ok(_) => println!("❌ PROBLEM: verb_definitions matched too much content!"),
        Err(_) => println!("✅ Good: verb_definitions correctly rejects extra content"),
    }
    
    // Test 3: Try to parse the sequence as separate rules
    println!("\n=== Test 3: Test if we can manually sequence the rules ===");
    
    let mut cursor = 0;
    let full_content = r#"2
do_start_script
3
173
-1
do_login_command
3
173
-1
0
0"#;
    
    println!("Full content to parse: '{}'", full_content);
    
    // Find where verb_definitions ends
    match LambdaMooDbParser::parse(Rule::verb_definitions, verb_section) {
        Ok(_) => {
            println!("✅ verb_definitions ends at position {}", verb_section.len());
            
            // Now try to parse property_definitions starting from the next character
            let remaining = &full_content[verb_section.len()..];
            println!("Remaining content: '{}'", remaining);
            
            // The remaining should be "\n0\n0" 
            // Let's try to parse "0\n" as property_definitions
            if remaining.starts_with('\n') {
                let prop_content = &remaining[1..]; // Skip the newline
                println!("Property content: '{}'", prop_content);
                
                // Now prop_content should be "0\n0"
                // Let's try to parse just "0\n" as property_definitions
                if let Some(first_line_end) = prop_content.find('\n') {
                    let prop_def_part = &prop_content[..first_line_end + 1]; // Include the newline
                    println!("Property definitions part: '{}'", prop_def_part);
                    
                    match LambdaMooDbParser::parse(Rule::property_definitions, prop_def_part) {
                        Ok(_) => println!("✅ property_definitions parsed successfully"),
                        Err(e) => println!("❌ property_definitions failed: {}", e),
                    }
                }
            }
        }
        Err(e) => println!("❌ Unexpected: verb_definitions failed: {}", e),
    }
    
    Ok(())
}