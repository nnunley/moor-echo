use std::fs;
use pest::Parser;
use echo_core::parser::lambdamoo_db_parser::{LambdaMooDbParser, Rule};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Debugging LambdaCore property section where error likely occurs...");
    
    let content = fs::read_to_string("examples/LambdaCore-latest.db")?;
    let lines: Vec<&str> = content.lines().collect();
    
    // Find object #0 and examine its property section  
    let mut obj_start = None;
    for (i, line) in lines.iter().enumerate() {
        if line.starts_with("#0") && !line.contains(":") {
            obj_start = Some(i);
            break;
        }
    }
    
    if let Some(start) = obj_start {
        println!("Object #0 starts at line {}", start + 1);
        
        // Based on previous analysis:
        // - Object header: line 11 (#0)
        // - Object fields: lines 12-21 (name through sibling)
        // - Verb section: lines 22-110 (22 verbs * 4 lines + 1 count line = 89 lines)
        // - Property section should start around line 111
        
        let verb_count = lines[start + 11].parse::<usize>().unwrap_or(0); // Line 22 has verb count
        let property_section_start = start + 11 + 1 + (verb_count * 4); // +11 for fields, +1 for verb count, +4*verbs
        
        println!("Property section should start around line {}", property_section_start + 1);
        
        // Show the property section structure
        println!("\nProperty section structure:");
        for i in 0..20 {
            if property_section_start + i < lines.len() {
                let line_num = property_section_start + i + 1;
                let line = lines[property_section_start + i];
                println!("{:3}: '{}'", line_num, line);
                
                if i == 0 {
                    println!("     ^ Should be property count");
                    match line.parse::<i64>() {
                        Ok(count) => println!("     ✅ Property count: {}", count),
                        Err(e) => println!("     ❌ Invalid property count: {}", e),
                    }
                } else {
                    println!("     ^ Property name or start of property values");
                }
            }
        }
        
        // Try to isolate the property definitions section
        let prop_count_line = property_section_start;
        if prop_count_line < lines.len() {
            let prop_count = lines[prop_count_line].parse::<usize>().unwrap_or(0);
            println!("\nProperty count: {}", prop_count);
            
            // Property definitions: count line + prop_count property names
            let prop_defs_end = prop_count_line + 1 + prop_count;
            
            if prop_defs_end <= lines.len() {
                let prop_defs_section: Vec<&str> = lines[prop_count_line..prop_defs_end].to_vec();
                let prop_defs_content = prop_defs_section.join("\n") + "\n";
                
                println!("\nProperty definitions section ({} lines):", prop_defs_section.len());
                for (i, line) in prop_defs_section.iter().enumerate() {
                    println!("  {}: '{}'", i, line);
                }
                
                match LambdaMooDbParser::parse(Rule::property_definitions, &prop_defs_content) {
                    Ok(_) => println!("✅ Property definitions parse successfully"),
                    Err(e) => println!("❌ Property definitions parsing failed: {}", e),
                }
                
                // Now check the property values section
                println!("\nProperty values section starts at line {}", prop_defs_end + 1);
                
                // Show first few lines of property values
                for i in 0..10 {
                    if prop_defs_end + i < lines.len() {
                        let line_num = prop_defs_end + i + 1;
                        let line = lines[prop_defs_end + i];
                        println!("{:3}: '{}'", line_num, line);
                        
                        if i == 0 {
                            println!("     ^ Should be property values count");
                            match line.parse::<i64>() {
                                Ok(count) => println!("     ✅ Property values count: {}", count),
                                Err(e) => println!("     ❌ Invalid property values count: {}", e),
                            }
                        }
                    }
                }
                
                // Try to parse a single property value to see if that's where the error is
                let prop_vals_start = prop_defs_end;
                if prop_vals_start + 4 < lines.len() {
                    // Try to parse first property value (should be 4 lines: count, type, content, owner, perms)
                    let single_propval_lines: Vec<&str> = lines[prop_vals_start..prop_vals_start + 5].to_vec();
                    let single_propval_content = single_propval_lines.join("\n") + "\n";
                    
                    println!("\nTrying to parse single property value:");
                    for (i, line) in single_propval_lines.iter().enumerate() {
                        println!("  {}: '{}'", i, line);
                    }
                    
                    // This might not work because property values have complex structure
                    // Let's just see what the structure looks like
                }
            }
        }
    }
    
    Ok(())
}