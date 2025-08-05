use pest::Parser;
use echo_core::parser::lambdamoo_db_parser::{LambdaMooDbParser, Rule};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing object structure parsing step by step...");
    
    // Test 1: object_name parsing
    println!("=== Test 1: object_name ===");
    match LambdaMooDbParser::parse(Rule::object_name, "System Object") {
        Ok(_) => println!("✅ object_name parsed"),
        Err(e) => println!("❌ object_name failed: {}", e),
    }
    
    // Test 2: object_handles parsing (empty)
    println!("\n=== Test 2: object_handles (empty) ===");
    match LambdaMooDbParser::parse(Rule::object_handles, "") {
        Ok(_) => println!("✅ object_handles (empty) parsed"),
        Err(e) => println!("❌ object_handles (empty) failed: {}", e),
    }
    
    // Test 3: Progressive object building
    println!("\n=== Test 3: Progressive object building ===");
    
    // Just name + handles + flags
    let partial1 = "System Object\n\n16";
    match LambdaMooDbParser::parse(Rule::lambdamoo_object_body, partial1) {
        Ok(_) => println!("✅ name+handles+flags parsed"),
        Err(e) => println!("❌ name+handles+flags failed: {}", e),
    }
    
    // Add more fields up to verb definitions
    let partial2 = "System Object\n\n16\n3\n-1\n-1\n-1\n1\n-1\n2";
    match LambdaMooDbParser::parse(Rule::lambdamoo_object_body, partial2) {
        Ok(_) => println!("✅ up to verb count parsed"),
        Err(e) => println!("❌ up to verb count failed: {}", e),
    }
    
    // Add verb definitions
    let partial3 = "System Object\n\n16\n3\n-1\n-1\n-1\n1\n-1\n2\n2\ndo_start_script\n3\n173\n-1\ndo_login_command\n3\n173\n-1";
    match LambdaMooDbParser::parse(Rule::lambdamoo_object_body, partial3) {
        Ok(_) => println!("✅ with verbs parsed"),
        Err(e) => println!("❌ with verbs failed: {}", e),
    }
    
    // Add property definitions (count 0)
    let partial4 = "System Object\n\n16\n3\n-1\n-1\n-1\n1\n-1\n2\n2\ndo_start_script\n3\n173\n-1\ndo_login_command\n3\n173\n-1\n0";
    match LambdaMooDbParser::parse(Rule::lambdamoo_object_body, partial4) {
        Ok(_) => println!("✅ with propdef count parsed"),
        Err(e) => println!("❌ with propdef count failed: {}", e),
    }
    
    // Add property values (count 0) - this is where it should complete
    let complete_object = "System Object\n\n16\n3\n-1\n-1\n-1\n1\n-1\n2\n2\ndo_start_script\n3\n173\n-1\ndo_login_command\n3\n173\n-1\n0\n0";
    match LambdaMooDbParser::parse(Rule::lambdamoo_object_body, complete_object) {
        Ok(mut pairs) => {
            if let Some(pair) = pairs.next() {
                println!("✅ Complete object parsed");
                println!("Consumed: {} chars", pair.as_str().len());
                println!("Expected: {} chars", complete_object.len());
                
                if pair.as_str().len() == complete_object.len() {
                    println!("✅ Perfect match!");
                } else {
                    println!("🚨 Length mismatch");
                }
            }
        }
        Err(e) => println!("❌ Complete object failed: {}", e),
    }
    
    // Test what happens when we add extra content after complete object
    println!("\n=== Test 4: Complete object + extra content ===");
    let with_extra = format!("{}\n#1\nRoot Class", complete_object);
    match LambdaMooDbParser::parse(Rule::lambdamoo_object_body, &with_extra) {
        Ok(mut pairs) => {
            if let Some(pair) = pairs.next() {
                println!("Consumed: {} chars out of {}", pair.as_str().len(), with_extra.len());
                
                if pair.as_str().len() > complete_object.len() {
                    println!("🚨 PROBLEM: Consumed extra content!");
                } else {
                    println!("✅ Stopped at object boundary");
                }
            }
        }
        Err(e) => println!("❌ With extra content failed: {}", e),
    }
    
    Ok(())
}