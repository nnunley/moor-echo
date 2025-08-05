use std::fs;
use pest::Parser;
use echo_core::parser::lambdamoo_db_parser::{LambdaMooDbParser, Rule};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Debugging property values boundary issue...");
    
    let content = fs::read_to_string("examples/LambdaCore-latest.db")?;
    let lines: Vec<&str> = content.lines().collect();
    
    // Look at the end of object #0 where it should stop
    // Object #0 starts at line 11, object #1 starts at line 821
    let obj0_start = 10; // 0-indexed
    let obj1_start = 820; // 0-indexed
    
    println!("Object #0 boundary analysis:");
    println!("Should end before line {}", obj1_start + 1);
    
    // Show lines around where object #0 should end
    println!("\nLines around the boundary (lines 810-825):");
    for i in 809..825 {
        if i < lines.len() {
            let marker = if i == obj1_start { " <- Object #1 starts here" } else { "" };
            println!("{:4}: '{}'{}", i + 1, lines[i], marker);
        }
    }
    
    // The issue is likely that property values are consuming too much
    // Let's look at the structure of property values in object #0
    
    // Try to find where property values start in object #0
    // Property values should be after property definitions
    // Look for the pattern: number (property count), followed by property names, 
    // then another number (property values count), followed by values
    
    println!("\nLooking for property values section in object #0...");
    
    // Start from object #0 and look for property sections
    let mut current_line = obj0_start + 11; // Skip object header and basic fields
    
    // Skip verb definitions first - find the verb count
    while current_line < obj1_start && !lines[current_line].chars().all(|c| c.is_ascii_digit()) {
        current_line += 1;
    }
    
    if current_line < obj1_start {
        let verb_count: usize = lines[current_line].parse().unwrap_or(0);
        println!("Found verb count at line {}: {}", current_line + 1, verb_count);
        current_line += 1;
        
        // Skip verb definitions (4 lines per verb: name, owner, perms, prep)
        current_line += verb_count * 4;
        
        // Now we should be at property definitions
        if current_line < obj1_start {
            let prop_def_count: usize = lines[current_line].parse().unwrap_or(0);
            println!("Found property definitions count at line {}: {}", current_line + 1, prop_def_count);
            current_line += 1;
            
            // Skip property definitions (1 line per property)
            current_line += prop_def_count;
            
            // Now we should be at property values
            if current_line < obj1_start {
                let prop_val_count: usize = lines[current_line].parse().unwrap_or(0);
                println!("Found property values count at line {}: {}", current_line + 1, prop_val_count);
                current_line += 1;
                
                println!("Property values section starts at line {}", current_line + 1);
                println!("Should contain {} property values", prop_val_count);
                
                // Each property value has: type, content, owner, perms (variable lines depending on type)
                // This is where the issue likely is - property values are consuming too much
                
                println!("\nFirst few property values:");
                let prop_val_start = current_line;
                for i in 0..10 {
                    if prop_val_start + i < obj1_start && prop_val_start + i < lines.len() {
                        println!("{:4}: '{}'", prop_val_start + i + 1, lines[prop_val_start + i]);
                    }
                }
                
                println!("\nLast few lines that should be property values:");
                for i in (obj1_start - 10)..obj1_start {
                    if i < lines.len() {
                        println!("{:4}: '{}'", i + 1, lines[i]);
                    }
                }
                
                // Calculate how many lines the property values section is consuming
                let prop_val_lines_consumed = obj1_start - current_line;
                println!("\nProperty values section consumes {} lines", prop_val_lines_consumed);
                println!("That's {} lines per property value on average", prop_val_lines_consumed as f32 / prop_val_count as f32);
            }
        }
    }
    
    Ok(())
}