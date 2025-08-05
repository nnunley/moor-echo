use pest::Parser;
use echo_core::parser::lambdamoo_db_parser::{LambdaMooDbParser, Rule};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing propval rule behavior...");
    
    // Test 1: What happens when we have propval_count = 0?
    println!("=== Test 1: property_values with count 0 ===");
    let content_with_count_0 = "0\n#1\nRoot Class";
    match LambdaMooDbParser::parse(Rule::property_values, content_with_count_0) {
        Ok(mut pairs) => {
            if let Some(pair) = pairs.next() {
                println!("✅ property_values parsed");
                println!("Consumed: '{}'", pair.as_str().replace('\n', "\\n"));
                println!("Length: {} out of {}", pair.as_str().len(), content_with_count_0.len());
                
                if pair.as_str().contains("#1") {
                    println!("🚨 PROBLEM: property_values consumed the next object header!");
                }
            }
        }
        Err(e) => println!("❌ property_values failed: {}", e),
    }
    
    // Test 2: Test the propval rule directly
    println!("\n=== Test 2: propval rule on object content ===");
    let object_header_content = "#1\nRoot Class\n\n16";
    match LambdaMooDbParser::parse(Rule::propval, object_header_content) {
        Ok(mut pairs) => {
            if let Some(pair) = pairs.next() {
                println!("🚨 PROBLEM: propval rule matched object header content!");
                println!("Consumed: '{}'", pair.as_str().replace('\n', "\\n"));
            }
        }
        Err(_) => println!("✅ Good: propval rule correctly rejected object header content"),
    }
    
    // Test 3: Test the value rule on object content
    println!("\n=== Test 3: value rule on object content ===");
    match LambdaMooDbParser::parse(Rule::value, object_header_content) {
        Ok(mut pairs) => {
            if let Some(pair) = pairs.next() {
                println!("🚨 PROBLEM: value rule matched object header content!");
                println!("Consumed: '{}'", pair.as_str().replace('\n', "\\n"));
            }
        }
        Err(_) => println!("✅ Good: value rule correctly rejected object header content"),
    }
    
    // Test 4: Test value_content rules on object content
    println!("\n=== Test 4: str_value rule on object content ===");
    match LambdaMooDbParser::parse(Rule::str_value, object_header_content) {
        Ok(mut pairs) => {
            if let Some(pair) = pairs.next() {
                println!("🚨 PROBLEM: str_value rule matched object header content!");
                println!("Consumed: '{}'", pair.as_str().replace('\n', "\\n"));
            }
        }
        Err(_) => println!("✅ Good: str_value rule correctly rejected object header content"),
    }
    
    // Test 5: Test raw_string on object content
    println!("\n=== Test 5: raw_string rule on object content ===");
    match LambdaMooDbParser::parse(Rule::raw_string, "#1") {
        Ok(mut pairs) => {
            if let Some(pair) = pairs.next() {
                println!("🚨 POTENTIAL ISSUE: raw_string matched '#1'");
                println!("Consumed: '{}'", pair.as_str());
                println!("This might be causing the greedy consumption");
            }
        }
        Err(_) => println!("✅ Good: raw_string correctly rejected '#1'"),
    }
    
    Ok(())
}