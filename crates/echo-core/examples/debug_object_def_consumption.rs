use std::fs;
use pest::Parser;
use echo_core::parser::lambdamoo_db_parser::{LambdaMooDbParser, Rule};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing object_def consumption to see why object_list stops after first object...");
    
    // Get the object section from Minimal.db
    let content = fs::read_to_string("examples/Minimal.db")?;
    let lines: Vec<&str> = content.lines().collect();
    
    // Find object bounds
    let mut objects_start = None;
    let mut verb_programs_start = None;
    
    for (i, line) in lines.iter().enumerate() {
        if line.starts_with("#") && !line.contains(":") && objects_start.is_none() {
            objects_start = Some(i);
        }
        if line.contains(":") && line.starts_with("#") {
            verb_programs_start = Some(i);
            break;
        }
    }
    
    if let (Some(start), Some(end)) = (objects_start, verb_programs_start) {
        let object_section = lines[start..end].join("\n");
        
        println!("Object section length: {} characters", object_section.len());
        println!("Object section lines: {}", end - start);
        
        // Test first object_def parsing to see how much it consumes
        println!("\n=== Testing first object_def consumption ===");
        match LambdaMooDbParser::parse(Rule::object_def, &object_section) {
            Ok(mut pairs) => {
                if let Some(pair) = pairs.next() {
                    let consumed = pair.as_str();
                    println!("✅ First object_def parsed");
                    println!("Consumed {} characters out of {}", consumed.len(), object_section.len());
                    println!("Consumed {} lines", consumed.lines().count());
                    
                    // Show what was consumed
                    println!("\nConsumed content:");
                    for (i, line) in consumed.lines().enumerate() {
                        println!("  {}: '{}'", i+1, line);
                    }
                    
                    // Show what remains
                    let remaining = &object_section[consumed.len()..];
                    println!("\nRemaining content ({} chars):", remaining.len());
                    if remaining.len() > 0 {
                        for (i, line) in remaining.lines().take(10).enumerate() {
                            println!("  {}: '{}'", i+1, line);
                        }
                        if remaining.lines().count() > 10 {
                            println!("  ... ({} more lines)", remaining.lines().count() - 10);
                        }
                    }
                    
                    // Try parsing the remaining content as another object_def
                    if remaining.trim().len() > 0 {
                        println!("\n=== Testing second object_def on remaining content ===");
                        match LambdaMooDbParser::parse(Rule::object_def, remaining.trim()) {
                            Ok(_) => println!("✅ Second object_def parses from remainder"),
                            Err(e) => println!("❌ Second object_def failed on remainder: {}", e),
                        }
                    }
                } else {
                    println!("❌ No pairs returned from object_def parsing");
                }
            }
            Err(e) => println!("❌ First object_def failed: {}", e),
        }
        
        // Also test if the issue is with object_list parsing multiple objects
        println!("\n=== Testing object_list detailed parsing ===");
        match LambdaMooDbParser::parse(Rule::object_list, &object_section) {
            Ok(mut pairs) => {
                if let Some(pair) = pairs.next() {
                    println!("✅ object_list parsed, consumed {} chars", pair.as_str().len());
                    
                    let mut obj_count = 0;
                    for inner in pair.into_inner() {
                        match inner.as_rule() {
                            Rule::object_count => {
                                println!("  Found object_count: {}", inner.as_str());
                            }
                            Rule::object_def => {
                                obj_count += 1;
                                let obj_content = inner.as_str();
                                println!("  Object {}: {} chars, first line: '{}'", 
                                    obj_count, 
                                    obj_content.len(),
                                    obj_content.lines().next().unwrap_or(""));
                            }
                            _ => {
                                println!("  Other rule: {:?}", inner.as_rule());
                            }
                        }
                    }
                    println!("Total objects in object_list: {}", obj_count);
                }
            }
            Err(e) => println!("❌ object_list failed: {}", e),
        }
        
    } else {
        println!("Could not find object boundaries");
    }
    
    Ok(())
}