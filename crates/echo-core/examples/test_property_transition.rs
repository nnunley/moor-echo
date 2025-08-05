use pest::Parser;
use echo_core::parser::lambdamoo_db_parser::{LambdaMooDbParser, Rule};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing property definitions -> property values transition...");
    
    // Test 1: Just property definitions with count 0
    println!("=== Test 1: Property definitions alone ===");
    let prop_defs = "0\n";
    match LambdaMooDbParser::parse(Rule::property_definitions, prop_defs) {
        Ok(mut pairs) => {
            if let Some(pair) = pairs.next() {
                println!("✅ property_definitions parsed, consumed: '{}'", pair.as_str());
                println!("   Length consumed: {}", pair.as_str().len());
            }
        }
        Err(e) => println!("❌ property_definitions failed: {}", e),
    }
    
    // Test 2: Just property values with count 0
    println!("\n=== Test 2: Property values alone ===");
    let prop_vals = "0\n";
    match LambdaMooDbParser::parse(Rule::property_values, prop_vals) {
        Ok(mut pairs) => {
            if let Some(pair) = pairs.next() {
                println!("✅ property_values parsed, consumed: '{}'", pair.as_str());
                println!("   Length consumed: {}", pair.as_str().len());
            }
        }
        Err(e) => println!("❌ property_values failed: {}", e),
    }
    
    // Test 3: Both together in sequence - this should work as consecutive rules
    println!("\n=== Test 3: Both property sections in sequence ===");
    let both_props = "0\n0\n";
    println!("Input: '{}'", both_props.replace('\n', "\\n"));
    
    // Can't test this directly as there's no combined rule, but let's try simulating the parsing
    
    // Test 4: Try parsing each section step by step manually
    println!("\n=== Test 4: Manual step-by-step parsing ===");
    let input = "0\n0\n";
    
    // Parse first section as property_definitions
    match LambdaMooDbParser::parse(Rule::property_definitions, "0\n") {
        Ok(_) => {
            println!("✅ First section (property_definitions) parses");
            
            // Parse second section as property_values  
            match LambdaMooDbParser::parse(Rule::property_values, "0\n") {
                Ok(_) => println!("✅ Second section (property_values) parses"),
                Err(e) => println!("❌ Second section (property_values) failed: {}", e),
            }
        }
        Err(e) => println!("❌ First section (property_definitions) failed: {}", e),
    }
    
    // Test 5: The exact failing sequence from the object body
    println!("\n=== Test 5: The exact failing sequence ===");
    let exact = r#"2
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
    
    println!("Testing this sequence as verb_definitions + property_definitions + property_values");
    
    // Try parsing just the end part after verbs
    let after_verbs = "0\n0";
    println!("Content after verbs: '{}'", after_verbs.replace('\n', "\\n"));
    
    // The issue might be that property_values expects a trailing newline
    // Let's check what property_values rule actually expects again
    
    println!("\n=== Test 6: Property values rule validation ===");
    
    // Test property_values with and without trailing newline
    match LambdaMooDbParser::parse(Rule::property_values, "0\n") {
        Ok(_) => println!("✅ property_values('0\\n') works"),
        Err(e) => println!("❌ property_values('0\\n') failed: {}", e),
    }
    
    match LambdaMooDbParser::parse(Rule::property_values, "0") {
        Ok(_) => println!("✅ property_values('0') works"),
        Err(e) => println!("❌ property_values('0') failed: {}", e),
    }
    
    Ok(())
}