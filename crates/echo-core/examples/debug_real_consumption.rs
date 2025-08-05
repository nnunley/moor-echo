use pest::Parser;
use echo_core::parser::lambdamoo_db_parser::{LambdaMooDbParser, Rule};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing consumption with real database structure...");
    
    // Real property_values section from object #0: has 0 properties
    println!("=== Test 1: Real property_values from object #0 ===");
    let real_prop_values = "0\n0\n#1\nRoot Class";
    
    match LambdaMooDbParser::parse(Rule::property_values, real_prop_values) {
        Ok(mut pairs) => {
            if let Some(pair) = pairs.next() {
                println!("property_values consumed: '{}'", pair.as_str().replace('\n', "\\n"));
                println!("Length: {} out of {}", pair.as_str().len(), real_prop_values.len());
                
                if pair.as_str().contains("#1") || pair.as_str().contains("Root Class") {
                    println!("🚨 SMOKING GUN: property_values consumed the next object!");
                } else {
                    println!("✅ property_values correctly stopped");
                }
            }
        }
        Err(e) => println!("❌ property_values failed: {}", e),
    }
    
    // Test what happens with the entire object #0
    println!("\n=== Test 2: Entire object #0 ===");  
    let object_0 = "System Object\n\n16\n3\n-1\n-1\n-1\n1\n-1\n2\n2\ndo_start_script\n3\n173\n-1\ndo_login_command\n3\n173\n-1\n0\n0\n#1\nRoot Class";
    
    match LambdaMooDbParser::parse(Rule::lambdamoo_object_body, object_0) {
        Ok(mut pairs) => {
            if let Some(pair) = pairs.next() {
                println!("object_body consumed: {} chars out of {}", pair.as_str().len(), object_0.len());
                
                if pair.as_str().contains("Root Class") {
                    println!("🚨 CRITICAL: object_body consumed next object!");
                    
                    // Let's see what the last few lines are
                    let lines: Vec<&str> = pair.as_str().lines().collect();
                    println!("Last few lines consumed:");
                    for line in lines.iter().rev().take(5).rev() {
                        println!("  '{}'", line);
                    }
                } else {
                    println!("✅ object_body correctly stopped");
                }
            }
        }
        Err(e) => println!("❌ object_body failed: {}", e),
    }
    
    // Test the exact failing case
    println!("\n=== Test 3: object_def with real data ===");
    let object_def_content = "#0\nSystem Object\n\n16\n3\n-1\n-1\n-1\n1\n-1\n2\n2\ndo_start_script\n3\n173\n-1\ndo_login_command\n3\n173\n-1\n0\n0\n#1\nRoot Class";
    
    match LambdaMooDbParser::parse(Rule::object_def, object_def_content) {
        Ok(mut pairs) => {
            if let Some(pair) = pairs.next() {
                println!("object_def consumed: {} chars out of {}", pair.as_str().len(), object_def_content.len());
                
                if pair.as_str().contains("Root Class") {
                    println!("🚨 CRITICAL: object_def consumed next object!");
                } else {
                    println!("✅ object_def correctly stopped");
                }
            }
        }
        Err(e) => println!("❌ object_def failed: {}", e),
    }
    
    Ok(())
}