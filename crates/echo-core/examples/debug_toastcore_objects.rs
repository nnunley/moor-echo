use std::fs;
use pest::Parser;
use echo_core::parser::lambdamoo_db_parser::{LambdaMooDbParser, Rule};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Debugging ToastCore object parsing...");
    
    let content = fs::read_to_string("examples/toastcore.db")?;
    println!("ToastCore database: {} chars, {} lines", content.len(), content.lines().count());
    
    // Look at the first 50 lines to understand the structure
    println!("\n=== First 50 lines of ToastCore ===");
    for (i, line) in content.lines().take(50).enumerate() {
        println!("{:3}: '{}'", i+1, line);
    }
    
    // Try to find object headers manually
    println!("\n=== Looking for object headers ===");
    let mut object_headers = Vec::new();
    for (i, line) in content.lines().enumerate() {
        if line.len() > 1 && line.starts_with('#') && !line.contains(':') {
            // Check if it's all digits after #
            if line[1..].chars().all(|c| c.is_ascii_digit() || c == '-') {
                object_headers.push((i+1, line.to_string()));
                if object_headers.len() <= 10 {  // Show first 10
                    println!("  Line {}: '{}'", i+1, line);
                }
            }
        }
    }
    println!("Total object headers found: {}", object_headers.len());
    
    if object_headers.len() > 0 {
        // Try parsing the database up to the first few objects
        let first_obj_line = object_headers[0].0;
        println!("\n=== Testing database parsing up to first object (line {}) ===", first_obj_line);
        
        let lines: Vec<&str> = content.lines().collect();
        let partial_content = lines[0..first_obj_line+20].join("\n") + "\n";  // Include 20 lines after first object
        
        match LambdaMooDbParser::parse(Rule::database, &partial_content) {
            Ok(mut pairs) => {
                if let Some(db_pair) = pairs.next() {
                    println!("✅ Partial ToastCore parsed successfully");
                    
                    // Find the object_list section
                    for inner in db_pair.into_inner() {
                        if let Rule::object_list = inner.as_rule() {
                            println!("Found object_list section");
                            println!("Content length: {} chars", inner.as_str().len());
                            
                            let inner_items: Vec<_> = inner.into_inner().collect();
                            println!("Inner items: {}", inner_items.len());
                            
                            for (i, item) in inner_items.iter().enumerate() {
                                match item.as_rule() {
                                    Rule::object_count => {
                                        println!("  Item {}: object_count = '{}'", i, item.as_str());
                                    }
                                    Rule::object_def => {
                                        println!("  Item {}: object_def = '{}'", i, 
                                            item.as_str().lines().next().unwrap_or(""));
                                    }
                                    _ => {
                                        println!("  Item {}: {:?}", i, item.as_rule());
                                    }
                                }
                            }
                            break;
                        }
                    }
                }
            }
            Err(e) => println!("❌ Partial ToastCore parsing failed: {}", e),
        }
    }
    
    Ok(())
}