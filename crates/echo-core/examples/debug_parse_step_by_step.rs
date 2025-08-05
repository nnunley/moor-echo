use std::fs;
use pest::Parser;
use echo_core::parser::lambdamoo_db_parser::{LambdaMooDbParser, Rule};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Step-by-step parse debugging...");
    
    // Get the exact structure from Minimal.db object #0
    let content = fs::read_to_string("../../examples/Minimal.db")?;
    let lines: Vec<&str> = content.lines().collect();
    
    // Extract just object #0 data - header through the two property counts
    let mut object_lines = Vec::new();
    let mut found_object_0 = false;
    let mut found_object_1 = false;
    
    for line in lines {
        if line == "#0" {
            found_object_0 = true;
            continue; // Skip the header for object body test
        }
        if line == "#1" {
            found_object_1 = true;
            break;
        }
        if found_object_0 && !found_object_1 {
            object_lines.push(line);
        }
    }
    
    let object_body = object_lines.join("\n");
    println!("Object #0 body content ({} lines):", object_lines.len());
    for (i, line) in object_lines.iter().enumerate() {
        println!("  {}: '{}'", i+1, line);
    }
    println!("");
    
    // Test individual components step by step
    
    // Step 1: Object name
    println!("=== Step 1: Object Name ===");
    let name_line = &object_lines[0];
    match LambdaMooDbParser::parse(Rule::object_name, name_line) {
        Ok(_) => println!("✅ object_name: '{}'", name_line),
        Err(e) => println!("❌ object_name failed: {}", e),
    }
    
    // Step 2: Object handles (empty line)
    println!("\n=== Step 2: Object Handles ===");
    let handles_line = &object_lines[1];
    match LambdaMooDbParser::parse(Rule::object_handles, handles_line) {
        Ok(_) => println!("✅ object_handles: '{}'", handles_line),
        Err(e) => println!("❌ object_handles failed: {}", e),
    }
    
    // Step 3: Try to parse up to verb definitions
    println!("\n=== Step 3: Up to Verb Definitions ===");
    let up_to_verbs = object_lines[0..11].join("\n") + "\n" + &object_lines[11..19].join("\n");
    println!("Content up to verbs: {}", up_to_verbs.chars().take(100).collect::<String>());
    
    // Step 4: Test verb definitions alone  
    println!("\n=== Step 4: Verb Definitions Alone ===");
    let verb_section = object_lines[10..19].join("\n"); // Include the verb count "2"
    println!("Verb section: '{}'", verb_section.chars().take(50).collect::<String>());
    match LambdaMooDbParser::parse(Rule::verb_definitions, &verb_section) {
        Ok(_) => println!("✅ verb_definitions parsed"),
        Err(e) => println!("❌ verb_definitions failed: {}", e),
    }
    
    // Step 5: Test property sections
    println!("\n=== Step 5: Property Definitions ===");
    let prop_def_line = &object_lines[19]; // Should be "0"
    match LambdaMooDbParser::parse(Rule::propdef_count, prop_def_line) {
        Ok(_) => println!("✅ propdef_count: '{}'", prop_def_line),
        Err(e) => println!("❌ propdef_count failed: {}", e),
    }
    
    println!("\n=== Step 6: Property Values Count ===");
    let prop_val_line = &object_lines[20]; // Should be "0"
    match LambdaMooDbParser::parse(Rule::propval_count, prop_val_line) {
        Ok(_) => println!("✅ propval_count: '{}'", prop_val_line),
        Err(e) => println!("❌ propval_count failed: {}", e),
    }
    
    // Step 7: Test complete property sections
    println!("\n=== Step 7: Complete Property Definitions ===");
    let prop_def_section = format!("{}\n", object_lines[19]); // "0\n"
    match LambdaMooDbParser::parse(Rule::property_definitions, &prop_def_section) {
        Ok(_) => println!("✅ property_definitions: '{}'", prop_def_section.trim()),
        Err(e) => println!("❌ property_definitions failed: {}", e),
    }
    
    println!("\n=== Step 8: Complete Property Values ===");
    let prop_val_section = format!("{}\n", object_lines[20]); // "0\n"  
    match LambdaMooDbParser::parse(Rule::property_values, &prop_val_section) {
        Ok(_) => println!("✅ property_values: '{}'", prop_val_section.trim()),
        Err(e) => println!("❌ property_values failed: {}", e),
    }
    
    // Step 9: Test the transition sequence 
    println!("\n=== Step 9: Verb->Property Transition ===");
    let transition = format!("{}\n{}\n{}", 
        object_lines[18], // "-1" (last verb prep)
        object_lines[19], // "0" (prop def count)
        object_lines[20]  // "0" (prop val count)
    );
    println!("Transition content: '{}'", transition);
    
    // Step 10: Complete object body test
    println!("\n=== Step 10: Complete Object Body (no extra newline) ===");
    match LambdaMooDbParser::parse(Rule::lambdamoo_object_body, &object_body) {
        Ok(_) => println!("✅ Complete lambdamoo_object_body parsed"),
        Err(e) => {
            println!("❌ Complete lambdamoo_object_body failed: {}", e);
            println!("Body length: {}", object_body.len());
            println!("Last 20 chars: {:?}", object_body.chars().rev().take(20).collect::<Vec<_>>());
            
            // Let me check if the issue is at the end - maybe property_values needs to end differently
            println!("Testing without final newline...");
            let body_no_final_newline = object_body.trim_end();
            match LambdaMooDbParser::parse(Rule::lambdamoo_object_body, body_no_final_newline) {
                Ok(_) => println!("✅ Without final newline: SUCCESS"),
                Err(e2) => println!("❌ Without final newline still failed: {}", e2),
            }
        }
    }
    
    Ok(())
}