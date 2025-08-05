use std::fs;
use pest::Parser;
use echo_core::parser::lambdamoo_db_parser::{LambdaMooDbParser, Rule};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing with exact object section from database...");
    
    // Read the exact object section that was extracted
    let object_section = fs::read_to_string("/tmp/object_section.txt")?;
    println!("Object section length: {} chars", object_section.len());
    println!("Object section lines: {}", object_section.lines().count());
    
    // Add the object count (4) that would be in the real object_list
    let object_list_content = format!("4\n{}", object_section);
    
    // Test 1: Parse as object_list  
    println!("\n=== Test 1: Parse as object_list ===");
    match LambdaMooDbParser::parse(Rule::object_list, &object_list_content) {
        Ok(mut pairs) => {
            if let Some(pair) = pairs.next() {
                println!("✅ object_list parsed successfully");
                println!("Consumed: {} chars", pair.as_str().len());
                
                let mut obj_count = 0;
                for inner in pair.into_inner() {
                    match inner.as_rule() {
                        Rule::object_def => {
                            obj_count += 1;
                            println!("  Object {} found", obj_count);
                        }
                        Rule::object_count => {
                            println!("  Object count: {}", inner.as_str());
                        }
                        _ => {}
                    }
                }
                println!("Total objects found: {}", obj_count);
            }
        }
        Err(e) => println!("❌ object_list failed: {}", e),
    }
    
    // Test 2: Parse first object_def directly with exact content
    println!("\n=== Test 2: Parse first object_def directly ===");
    let lines: Vec<&str> = object_section.lines().collect();
    
    // Find where the first object ends - should be before #1
    let mut first_obj_end = None;
    for (i, line) in lines.iter().enumerate() {
        if line.starts_with("#1") {
            first_obj_end = Some(i);
            break;
        }
    }
    
    if let Some(end) = first_obj_end {
        println!("First object should end at line {} (0-indexed)", end);
        let first_object_content = lines[0..end].join("\n") + "\n";
        println!("Expected first object length: {} chars", first_object_content.len());
        
        // Add the #0 header  
        let full_first_object = format!("#0\n{}", first_object_content);
        
        match LambdaMooDbParser::parse(Rule::object_def, &full_first_object) {
            Ok(mut pairs) => {
                if let Some(pair) = pairs.next() {
                    println!("✅ First object_def parsed");
                    println!("Consumed: {} chars", pair.as_str().len());
                    println!("Expected: {} chars", full_first_object.len());
                    
                    if pair.as_str().len() == full_first_object.len() {
                        println!("✅ Consumed exactly the right amount");
                    } else {
                        println!("🚨 Size mismatch!");
                    }
                }
            }
            Err(e) => println!("❌ First object_def failed: {}", e),
        }
    }
    
    Ok(())
}