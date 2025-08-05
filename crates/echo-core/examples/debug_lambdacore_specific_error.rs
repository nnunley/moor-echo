use std::fs;
use pest::Parser;
use echo_core::parser::lambdamoo_db_parser::{LambdaMooDbParser, Rule};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Finding exact field causing LambdaCore parsing error...");
    
    let content = fs::read_to_string("examples/LambdaCore-latest.db")?;
    let lines: Vec<&str> = content.lines().collect();
    
    // Look at the structure around object #0 (The System Object)
    println!("\n=== First object in LambdaCore ===");
    
    // Find the first object header
    let mut obj_start = None;
    for (i, line) in lines.iter().enumerate() {
        if line.starts_with("#0") && !line.contains(":") {
            obj_start = Some(i);
            println!("Found object #0 at line {}: '{}'", i+1, line);
            break;
        }
    }
    
    if let Some(start) = obj_start {
        // Show the object structure - first 20 lines after the object header
        println!("\nFirst 20 lines of object #0:");
        for i in 0..20 {
            if start + i < lines.len() {
                let line_num = start + i + 1;
                let line = lines[start + i];
                println!("{:3}: '{}'", line_num, line);
                
                // Check if this line looks problematic for numeric parsing
                if line.trim().is_empty() {
                    println!("     ^ EMPTY LINE - could cause 'invalid digit' error");
                } else if line.contains(" ") && !line.starts_with("\"") {
                    println!("     ^ Contains spaces - might not be expected");
                } else if !line.chars().all(|c| c.is_ascii_digit() || c == '-') && !line.starts_with("\"") {
                    println!("     ^ Non-numeric content: might cause parsing error");
                }
            }
        }
        
        // Try to parse just the object header to see if it works
        let obj_header_line = format!("{}\n", lines[start]);
        match LambdaMooDbParser::parse(Rule::object_header, &obj_header_line) {
            Ok(_) => println!("\n✅ Object header parses correctly"),
            Err(e) => println!("\n❌ Object header parsing failed: {}", e),
        }
        
        // Try to parse the object name (should be line after header)
        if start + 1 < lines.len() {
            let name_line = lines[start + 1];
            println!("\nObject name line: '{}'", name_line);
            if name_line.chars().any(|c| c.is_ascii_digit()) {
                println!("WARNING: Object name contains digits - might be confused with numeric field");
            }
        }
        
        // Check for common problematic patterns
        println!("\n=== Checking for common error patterns ===");
        for i in 1..15 {  // Check lines that should be numeric fields
            if start + i < lines.len() {
                let line = lines[start + i];
                let line_num = start + i + 1;
                
                // This should be a numeric field based on LambdaMOO format
                let field_name = match i {
                    1 => "name",
                    2 => "handles", 
                    3 => "flags",
                    4 => "owner",
                    5 => "location", 
                    6 => "contents",
                    7 => "next",
                    8 => "parent",
                    9 => "child",
                    10 => "sibling",
                    _ => "unknown"
                };
                
                if i >= 3 && i <= 10 {  // These should be numeric fields
                    match line.trim().parse::<i64>() {
                        Ok(val) => println!("Line {}: {} = {} ✅", line_num, field_name, val),
                        Err(e) => {
                            println!("Line {}: {} = '{}' ❌ PARSE ERROR: {}", line_num, field_name, line, e);
                            println!("  ^ THIS IS LIKELY THE CAUSE OF THE ERROR");
                        }
                    }
                } else {
                    println!("Line {}: {} = '{}'", line_num, field_name, line);
                }
            }
        }
    }
    
    Ok(())
}