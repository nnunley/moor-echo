use std::fs;
use pest::Parser;
use echo_core::parser::lambdamoo_db_parser::{LambdaMooDbParser, Rule};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Debugging LambdaCore first complete object parsing...");
    
    let content = fs::read_to_string("examples/LambdaCore-latest.db")?;
    let lines: Vec<&str> = content.lines().collect();
    
    // Find the complete first object (from #0 to #1)
    let mut obj_start = None;
    let mut obj_end = None;
    
    for (i, line) in lines.iter().enumerate() {
        if line.starts_with("#0") && !line.contains(":") {
            obj_start = Some(i);
            println!("Found object #0 at line {}", i + 1);
        } else if obj_start.is_some() && line.starts_with("#1") && !line.contains(":") {
            obj_end = Some(i);
            println!("Found object #1 at line {}", i + 1);
            break;
        }
    }
    
    if let (Some(start), Some(end)) = (obj_start, obj_end) {
        let object_lines: Vec<&str> = lines[start..end].to_vec();
        let object_content = object_lines.join("\n") + "\n";
        
        println!("\nFirst object is {} lines long", object_lines.len());
        println!("Lines {}-{}", start + 1, end);
        
        // Show a sample of the object content
        println!("\nFirst 10 lines of object #0:");
        for (i, line) in object_lines.iter().take(10).enumerate() {
            println!("{:3}: '{}'", start + i + 1, line);
        }
        
        println!("\nLast 10 lines of object #0:");
        let total_lines = object_lines.len();
        for (i, line) in object_lines.iter().skip(total_lines.saturating_sub(10)).enumerate() {
            println!("{:3}: '{}'", start + total_lines.saturating_sub(10) + i + 1, line);
        }
        
        // Try to parse the complete object
        println!("\nTrying to parse complete object #0...");
        
        match LambdaMooDbParser::parse(Rule::object_def, &object_content) {
            Ok(parsed) => {
                println!("✅ Complete object #0 parses successfully with grammar");
                
                // Now try to actually parse it with the Rust parser to see where it fails
                println!("\nTrying to parse object #0 with Rust parser...");
                
                for pair in parsed {
                    match echo_core::parser::lambdamoo_db_parser::LambdaMooDbParser::parse_object(pair) {
                        Ok(obj) => {
                            println!("✅ Object #0 parsed successfully with Rust parser!");
                            println!("   ID: {}", obj.id);
                            println!("   Name: '{}'", obj.name);
                            println!("   Flags: {}", obj.flags);
                            println!("   Owner: {}", obj.owner);
                            println!("   Verbs: {} defined", obj.verbs.len());
                            println!("   Properties: {} defined", obj.properties.len());
                            println!("   Property values: {} defined", obj.property_values.len());
                        }
                        Err(e) => {
                            println!("❌ Object #0 Rust parsing failed: {}", e);
                            println!("   Error details: {:?}", e);
                            
                            // This is likely where our "invalid digit found in string" error occurs
                            let error_str = format!("{}", e);
                            if error_str.contains("invalid digit") {
                                println!("\n🎯 FOUND THE ERROR!");
                                println!("   The error is in the Rust parsing code, not the grammar");
                                println!("   Likely caused by trying to parse a non-numeric string as a number");
                                
                                // Check if it's in the error chain for more specific info
                                let mut current_error: &dyn std::error::Error = &e;
                                let mut error_level = 0;
                                while let Some(source) = current_error.source() {
                                    error_level += 1;
                                    println!("   Error level {}: {}", error_level, source);
                                    current_error = source;
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => {
                println!("❌ Complete object #0 grammar parsing failed: {}", e);
            }
        }
    }
    
    Ok(())
}