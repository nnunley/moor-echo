use pest::Parser;
use echo_core::parser::lambdamoo_db_parser::{LambdaMooDbParser, Rule};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing property sections parsing...");
    
    // Test 1: property_definitions with count 0
    println!("=== Test 1: property_definitions with count 0 ===");
    let propdef_section = "0";
    match LambdaMooDbParser::parse(Rule::property_definitions, propdef_section) {
        Ok(mut pairs) => {
            if let Some(pair) = pairs.next() {
                println!("✅ property_definitions parsed: '{}'", pair.as_str());
                println!("Length: {}", pair.as_str().len());
            }
        }
        Err(e) => println!("❌ property_definitions failed: {}", e),
    }
    
    // Test 2: property_definitions followed by property_values  
    println!("\n=== Test 2: property_definitions + property_values ===");
    let both_sections = "0\n0";
    
    // Test just property_definitions part
    match LambdaMooDbParser::parse(Rule::property_definitions, both_sections) {
        Ok(mut pairs) => {
            if let Some(pair) = pairs.next() {
                println!("property_definitions consumed: '{}' (length {})", 
                    pair.as_str().replace('\n', "\\n"), pair.as_str().len());
                
                if pair.as_str().len() > 2 {
                    println!("🚨 property_definitions consumed too much!");
                }
            }
        }
        Err(e) => println!("❌ property_definitions on both failed: {}", e),
    }
    
    // Test 3: property_values with count 0
    println!("\n=== Test 3: property_values with count 0 ===");
    let propval_section = "0";
    match LambdaMooDbParser::parse(Rule::property_values, propval_section) {
        Ok(mut pairs) => {
            if let Some(pair) = pairs.next() {
                println!("✅ property_values parsed: '{}'", pair.as_str());
                println!("Length: {}", pair.as_str().len());
            }
        }
        Err(e) => println!("❌ property_values failed: {}", e),
    }
    
    // Test 4: The problematic sequence - what happens after property_values?
    println!("\n=== Test 4: property_values followed by object header ===");
    let propval_with_next = "0\n#1\nRoot Class";
    match LambdaMooDbParser::parse(Rule::property_values, propval_with_next) {
        Ok(mut pairs) => {
            if let Some(pair) = pairs.next() {
                println!("property_values consumed: '{}' (length {})", 
                    pair.as_str().replace('\n', "\\n"), pair.as_str().len());
                
                if pair.as_str().contains("#1") || pair.as_str().contains("Root Class") {
                    println!("🚨 CRITICAL: property_values consumed next object!");
                } else {
                    println!("✅ property_values stopped correctly");
                }
            }
        }
        Err(e) => println!("❌ property_values with next failed: {}", e),
    }
    
    // Test 5: What about the sequence: propdef_count, propval_count, next object?
    println!("\n=== Test 5: Complete property sections + next object ===");
    let complete_prop_section = "0\n0\n#1\nRoot Class";
    
    // This should parse as: property_definitions consumes "0\n", then property_values consumes "0\n", leaving "#1\nRoot Class" 
    
    println!("Testing property_definitions on complete section:");
    match LambdaMooDbParser::parse(Rule::property_definitions, complete_prop_section) {
        Ok(mut pairs) => {
            if let Some(pair) = pairs.next() {
                println!("  property_definitions consumed: '{}' (length {})", 
                    pair.as_str().replace('\n', "\\n"), pair.as_str().len());
            }
        }
        Err(e) => println!("  ❌ property_definitions failed: {}", e),
    }
    
    Ok(())
}