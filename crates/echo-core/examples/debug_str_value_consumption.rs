use pest::Parser;
use echo_core::parser::lambdamoo_db_parser::{LambdaMooDbParser, Rule};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing str_value consumption in property values context...");
    
    // Test 1: What happens when we have a string property value followed by object header?
    println!("=== Test 1: value with TYPE_STR followed by object header ===");
    let value_content = "1\nSome string value\n#1\nRoot Class";
    
    match LambdaMooDbParser::parse(Rule::value, value_content) {
        Ok(mut pairs) => {
            if let Some(pair) = pairs.next() {
                println!("✅ value parsed");
                println!("Consumed: '{}'", pair.as_str().replace('\n', "\\n"));
                println!("Length: {} out of {}", pair.as_str().len(), value_content.len());
                
                if pair.as_str().contains("#1") {
                    println!("🚨 CRITICAL: value rule consumed the next object header!");
                    println!("This is why object_def consumes multiple objects!");
                }
            }
        }
        Err(e) => println!("❌ value failed: {}", e),
    }
    
    // Test 2: What if the string value itself looks like an object ID?
    println!("\n=== Test 2: value with string that looks like object ID ===");
    let tricky_value = "1\n#123\n#1\nRoot Class";
    
    match LambdaMooDbParser::parse(Rule::value, tricky_value) {
        Ok(mut pairs) => {
            if let Some(pair) = pairs.next() {
                println!("Value consumed: '{}'", pair.as_str().replace('\n', "\\n"));
                if pair.as_str().contains("Root Class") {
                    println!("🚨 CRITICAL: Even worse - consumed way too much!");
                }
            }
        }
        Err(e) => println!("❌ tricky value failed: {}", e),
    }
    
    // Test 3: Let's see how property values with string content behaves
    println!("\n=== Test 3: property_values with actual string value ===");
    let prop_values_content = "1\n1\nSome property value\n#0\n5\n#1\nRoot Class";
    
    match LambdaMooDbParser::parse(Rule::property_values, prop_values_content) {
        Ok(mut pairs) => {
            if let Some(pair) = pairs.next() {
                println!("property_values consumed: '{}'", pair.as_str().replace('\n', "\\n"));
                println!("Length: {} out of {}", pair.as_str().len(), prop_values_content.len());
                
                if pair.as_str().contains("Root Class") {
                    println!("🚨 SMOKING GUN: property_values consumed the next object!");
                    println!("This explains why object_def consumes multiple objects!");
                }
            }
        }
        Err(e) => println!("❌ property_values failed: {}", e),
    }
    
    Ok(())
}