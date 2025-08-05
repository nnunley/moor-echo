use std::fs;
use pest::Parser;
use echo_core::parser::lambdamoo_db_parser::{LambdaMooDbParser, Rule};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Debugging why LambdaCore only parses 1 object...");
    
    let content = fs::read_to_string("examples/LambdaCore-latest.db")?;
    let lines: Vec<&str> = content.lines().collect();
    
    println!("LambdaCore has {} lines total", lines.len());
    
    // Find all object headers in the file
    let mut object_headers = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        if line.starts_with("#") && !line.contains(":") && line.len() > 1 {
            if let Ok(obj_id) = line[1..].parse::<i64>() {
                object_headers.push((i + 1, obj_id, line));
            }
        }
    }
    
    println!("Found {} object headers in LambdaCore:", object_headers.len());
    for (line_num, obj_id, header) in object_headers.iter().take(10) {
        println!("  Line {}: {} (object #{})", line_num, header, obj_id);
    }
    
    if object_headers.len() > 10 {
        println!("  ... and {} more objects", object_headers.len() - 10);
    }
    
    // Check what happens when we try to parse just the object list section
    // Find where objects start and end
    let objects_start = object_headers.first().map(|(line_num, _, _)| line_num - 1).unwrap_or(0);
    
    // Try to find where objects section ends by looking for verb programs section
    let mut objects_end = lines.len();
    for (i, line) in lines.iter().enumerate().skip(objects_start) {
        // Look for the start of verb programs (typically starts with a number on its own line
        // followed by verb program headers like "#0:verbname")
        if line.chars().all(|c| c.is_ascii_digit()) && i + 1 < lines.len() {
            let next_line = lines[i + 1];
            if next_line.starts_with("#") && next_line.contains(":") {
                objects_end = i;
                println!("Objects section likely ends at line {} before verb programs", i + 1);
                break;
            }
        }
    }
    
    println!("Objects section: lines {} to {}", objects_start + 1, objects_end + 1);
    println!("Objects section has {} lines", objects_end - objects_start);
    
    // Extract just the objects section and try to parse it
    let objects_section: Vec<&str> = lines[objects_start..objects_end].to_vec();
    let objects_content = objects_section.join("\n") + "\n";
    
    println!("\nTrying to parse objects section with {} lines...", objects_section.len());
    
    match LambdaMooDbParser::parse(Rule::object_list, &objects_content) {
        Ok(parsed) => {
            println!("✅ Objects section parses successfully with grammar");
            
            // Count how many object_def rules we find
            let mut object_count = 0;
            for pair in parsed {
                for inner in pair.into_inner() {
                    if let Rule::object_def = inner.as_rule() {
                        object_count += 1;
                    }
                }
            }
            println!("Grammar found {} object definitions", object_count);
        }
        Err(e) => {
            println!("❌ Objects section parsing failed: {}", e);
        }
    }
    
    Ok(())
}