use std::fs;
use pest::Parser;
use echo_core::parser::lambdamoo_db_parser::{LambdaMooDbParser, Rule};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Analyzing all objects structure to understand the pattern...");
    
    let content = fs::read_to_string("examples/Minimal.db")?;
    let lines: Vec<&str> = content.lines().collect();
    
    // Extract each object manually based on headers
    let objects = [
        (0, lines[6..28].to_vec()),   // Object #0: lines 7-28 (0-indexed 6-27)
        (1, lines[28..42].to_vec()),  // Object #1: lines 29-42 (0-indexed 28-41) 
        (2, lines[42..60].to_vec()),  // Object #2: lines 43-60 (0-indexed 42-59)
        (3, lines[60..74].to_vec()),  // Object #3: lines 61-74 (0-indexed 60-73)
    ];
    
    for (obj_num, obj_lines) in objects.iter() {
        println!("\n=== Object #{} Analysis ===", obj_num);
        println!("Lines: {}", obj_lines.len());
        
        // Show the structure
        for (i, line) in obj_lines.iter().enumerate() {
            println!("  {:2}: '{}'", i+1, line);
        }
        
        // Identify the structure pattern
        if obj_lines.len() >= 12 {
            println!("\nStructure analysis:");
            println!("  Header: {}", obj_lines[0]);
            println!("  Name: {}", obj_lines[1]);
            println!("  Handles: '{}'", obj_lines[2]);
            println!("  Fields: {} {} {} {} {} {} {} {}", 
                obj_lines[3], obj_lines[4], obj_lines[5], obj_lines[6], 
                obj_lines[7], obj_lines[8], obj_lines[9], obj_lines[10]);
            
            // Find verb count
            let verb_count_line = 11;
            if verb_count_line < obj_lines.len() {
                let verb_count: i32 = obj_lines[verb_count_line].parse().unwrap_or(-1);
                println!("  Verb count: {}", verb_count);
                
                // Calculate where properties start
                let props_start = verb_count_line + 1 + (verb_count * 4) as usize;
                if props_start < obj_lines.len() {
                    println!("  Properties start at line: {}", props_start + 1);
                    if props_start < obj_lines.len() {
                        let propdef_count: i32 = obj_lines[props_start].parse().unwrap_or(-1);
                        println!("  Propdef count: {}", propdef_count);
                        
                        let propval_start = props_start + 1 + propdef_count as usize;
                        if propval_start < obj_lines.len() {
                            let propval_count: i32 = obj_lines[propval_start].parse().unwrap_or(-1);
                            println!("  Propval count: {}", propval_count);
                            
                            // Show remaining lines after propval count
                            let remaining_start = propval_start + 1;
                            if remaining_start < obj_lines.len() {
                                println!("  Remaining lines after propval count:");
                                for (i, line) in obj_lines[remaining_start..].iter().enumerate() {
                                    println!("    {}: '{}'", remaining_start + i + 1, line);
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // Test parsing each object
        let obj_content = obj_lines.join("\n");
        match LambdaMooDbParser::parse(Rule::object_def, &obj_content) {
            Ok(_) => println!("✅ Object #{} parses successfully", obj_num),
            Err(e) => println!("❌ Object #{} parsing failed: {}", obj_num, e),
        }
    }
    
    Ok(())
}