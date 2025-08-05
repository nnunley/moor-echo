use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Comparing database structures to understand parsing differences...");
    
    // Read both databases
    let minimal_content = fs::read_to_string("examples/Minimal.db")?;
    let lambdacore_content = fs::read_to_string("examples/LambdaCore-latest.db")?;
    
    let minimal_lines: Vec<&str> = minimal_content.lines().collect();
    let lambdacore_lines: Vec<&str> = lambdacore_content.lines().collect();
    
    println!("Minimal MOO: {} lines", minimal_lines.len());
    println!("LambdaCore: {} lines", lambdacore_lines.len());
    
    // Compare headers
    println!("\n=== Headers ===");
    println!("Minimal:    '{}'", minimal_lines[0]);
    println!("LambdaCore: '{}'", lambdacore_lines[0]);
    
    // Compare intro blocks
    println!("\n=== Intro blocks ===");
    for i in 1..5 {
        if i < minimal_lines.len() && i < lambdacore_lines.len() {
            println!("Line {}: Minimal='{}', LambdaCore='{}'", i+1, minimal_lines[i], lambdacore_lines[i]);
        }
    }
    
    // Find first objects in both
    println!("\n=== First Objects ===");
    
    // Find first object in Minimal
    let mut minimal_obj_start = None;
    for (i, line) in minimal_lines.iter().enumerate() {
        if line.starts_with("#0") && !line.contains(":") {
            minimal_obj_start = Some(i);
            break;
        }
    }
    
    // Find first object in LambdaCore
    let mut lambdacore_obj_start = None;
    for (i, line) in lambdacore_lines.iter().enumerate() {
        if line.starts_with("#0") && !line.contains(":") {
            lambdacore_obj_start = Some(i);
            break;
        }
    }
    
    if let (Some(min_start), Some(lc_start)) = (minimal_obj_start, lambdacore_obj_start) {
        println!("Minimal object #0 starts at line {}", min_start + 1);
        println!("LambdaCore object #0 starts at line {}", lc_start + 1);
        
        // Compare first 15 lines of each object
        println!("\nFirst 15 lines of each object:");
        for i in 0..15 {
            let min_line = if min_start + i < minimal_lines.len() {
                minimal_lines[min_start + i]
            } else {
                "(end of file)"
            };
            
            let lc_line = if lc_start + i < lambdacore_lines.len() {
                lambdacore_lines[lc_start + i]
            } else {
                "(end of file)"
            };
            
            println!("{:2}: Min='{}' | LC='{}'", i, min_line, lc_line);
            
            // Highlight differences
            if min_line != lc_line {
                if i == 2 && min_line.trim().is_empty() && lc_line.trim().is_empty() {
                    // Both are empty handles - OK
                } else if i >= 3 && i <= 10 {
                    // These should be numeric fields - check if they're valid numbers
                    let min_is_num = min_line.parse::<i64>().is_ok();
                    let lc_is_num = lc_line.parse::<i64>().is_ok();
                    
                    if !min_is_num || !lc_is_num {
                        println!("    ^^^ POTENTIAL ISSUE: Non-numeric in numeric field!");
                        println!("        Min is number: {}, LC is number: {}", min_is_num, lc_is_num);
                    }
                }
            }
        }
    }
    
    // Check if there are any empty fields where numbers are expected
    println!("\n=== Checking for problematic patterns ===");
    
    if let Some(lc_start) = lambdacore_obj_start {
        // Check the standard numeric fields (flags, owner, location, contents, next, parent, child, sibling)
        let field_names = ["header", "name", "handles", "flags", "owner", "location", "contents", "next", "parent", "child", "sibling"];
        
        for i in 0..field_names.len() {
            if lc_start + i < lambdacore_lines.len() {
                let line = lambdacore_lines[lc_start + i];
                let field_name = field_names[i];
                
                if i >= 3 && i <= 10 {  // These should be numeric
                    match line.parse::<i64>() {
                        Ok(val) => println!("  {}: {} ✅", field_name, val),
                        Err(_) => {
                            println!("  {}: '{}' ❌ NOT A NUMBER!", field_name, line);
                            
                            // Check if it's empty or contains non-digit chars
                            if line.trim().is_empty() {
                                println!("    ^ EMPTY STRING - this will cause 'invalid digit' error");
                            } else if line.chars().any(|c| !c.is_ascii_digit() && c != '-') {
                                println!("    ^ CONTAINS NON-DIGITS - this will cause 'invalid digit' error");
                            }
                        }
                    }
                } else {
                    println!("  {}: '{}' (string field)", field_name, line);
                }
            }
        }
    }
    
    Ok(())
}