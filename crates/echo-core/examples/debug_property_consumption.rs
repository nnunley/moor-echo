use std::fs;
use pest::Parser;
use echo_core::parser::lambdamoo_db_parser::{LambdaMooDbParser, Rule};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing property section consumption...");
    
    // Get just the content from the first object's end through the second object's start
    let content = fs::read_to_string("examples/Minimal.db")?;
    let lines: Vec<&str> = content.lines().collect();
    
    // Find the first object's property sections and the next object
    let relevant_lines = &lines[26..40]; // Lines around object #0 properties and object #1 start
    
    for (i, line) in relevant_lines.iter().enumerate() {
        println!("Line {}: '{}'", i+20, line); // Show actual line numbers offset
    }
    
    // Test property_definitions on just "0"
    println!("\n=== Testing property_definitions on '0' ===");
    match LambdaMooDbParser::parse(Rule::property_definitions, "0\n") {
        Ok(mut pairs) => {
            if let Some(pair) = pairs.next() {
                println!("✅ property_definitions consumed: '{}'", pair.as_str().replace('\n', "\\n"));
                println!("Length: {}", pair.as_str().len());
            }
        }
        Err(e) => println!("❌ property_definitions failed: {}", e),
    }
    
    // Test property_values on just "0" 
    println!("\n=== Testing property_values on '0' ===");
    match LambdaMooDbParser::parse(Rule::property_values, "0\n") {
        Ok(mut pairs) => {
            if let Some(pair) = pairs.next() {
                println!("✅ property_values consumed: '{}'", pair.as_str().replace('\n', "\\n"));
                println!("Length: {}", pair.as_str().len());
            }
        }
        Err(e) => println!("❌ property_values failed: {}", e),
    }
    
    // Test what happens when we give property_values the content "0\n#1\n"
    println!("\n=== Testing property_values consumption with following object ===");
    let test_content = "0\n#1\n";
    match LambdaMooDbParser::parse(Rule::property_values, test_content) {
        Ok(mut pairs) => {
            if let Some(pair) = pairs.next() {
                println!("✅ property_values consumed: '{}'", pair.as_str().replace('\n', "\\n"));
                println!("Length: {} out of {} total", pair.as_str().len(), test_content.len());
                
                if pair.as_str().len() > 2 {
                    println!("🚨 PROBLEM: property_values consumed more than just '0\\n'!");
                }
            }
        }
        Err(e) => println!("❌ property_values failed: {}", e),
    }
    
    // Test the exact sequence: property sections + next object header
    println!("\n=== Testing property sequence with next object ===");
    let sequence = "0\n0\n#1\n";
    println!("Testing sequence: '{}'", sequence.replace('\n', "\\n"));
    
    // First test property_definitions
    match LambdaMooDbParser::parse(Rule::property_definitions, "0\n") {
        Ok(_) => {
            println!("✅ property_definitions('0\\n') OK");
            
            // Then test property_values on the remaining
            match LambdaMooDbParser::parse(Rule::property_values, "0\n") {
                Ok(_) => println!("✅ property_values('0\\n') OK"),
                Err(e) => println!("❌ property_values('0\\n') failed: {}", e),
            }
        }
        Err(e) => println!("❌ property_definitions('0\\n') failed: {}", e),
    }
    
    Ok(())
}