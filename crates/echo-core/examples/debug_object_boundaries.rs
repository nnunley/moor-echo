use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Debugging object boundaries in LambdaCore...");
    
    let content = fs::read_to_string("examples/LambdaCore-latest.db")?;
    let lines: Vec<&str> = content.lines().collect();
    
    // Find first few object headers
    let mut object_positions = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        if line.starts_with("#") && !line.contains(":") && line.len() > 1 {
            if let Ok(obj_id) = line[1..].parse::<i64>() {
                object_positions.push((i, obj_id, line));
                if object_positions.len() >= 3 {
                    break;
                }
            }
        }
    }
    
    println!("First 3 objects:");
    for (line_idx, obj_id, header) in &object_positions {
        println!("  Object #{} at line {}: '{}'", obj_id, line_idx + 1, header);
    }
    
    // Show the boundary between object #0 and object #1
    let obj0_start = object_positions[0].0;
    let obj1_start = object_positions[1].0;
    
    println!("\n=== Object #0 (lines {}-{}) ===", obj0_start + 1, obj1_start);
    println!("Object #0 has {} lines", obj1_start - obj0_start);
    
    // Show last 10 lines of object #0
    let obj0_end_start = (obj1_start - 10).max(obj0_start + 1);
    println!("\nLast 10 lines of object #0:");
    for i in obj0_end_start..obj1_start {
        println!("{:4}: '{}'", i + 1, lines[i]);
    }
    
    // Show first few lines of object #1
    println!("\nFirst 10 lines of object #1:");
    for i in obj1_start..obj1_start + 10 {
        if i < lines.len() {
            println!("{:4}: '{}'", i + 1, lines[i]);
        }
    }
    
    // Check what type of content is right before object #1
    let line_before_obj1 = lines[obj1_start - 1];
    println!("\nLine right before object #1 (line {}): '{}'", obj1_start, line_before_obj1);
    
    // Check if this looks like the end of property values
    if line_before_obj1.chars().all(|c| c.is_ascii_digit()) {
        println!("^ This looks like a numeric value (likely property permission)");
    } else if line_before_obj1.trim().is_empty() {
        println!("^ This is an empty line");
    } else {
        println!("^ This is some other content: {}", line_before_obj1);
    }
    
    Ok(())
}