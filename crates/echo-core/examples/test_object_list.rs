use std::fs;
use pest::Parser;
use echo_core::parser::lambdamoo_db_parser::{LambdaMooDbParser, Rule};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing object_list parsing...");
    
    // Read Minimal.db and extract the object section
    let content = fs::read_to_string("examples/Minimal.db")?;
    let lines: Vec<&str> = content.lines().collect();
    
    println!("Total lines in database: {}", lines.len());
    
    // Find where objects start (after player list)
    let mut objects_start = None;
    let mut verb_programs_start = None;
    
    for (i, line) in lines.iter().enumerate() {
        if line.starts_with("#") && !line.contains(":") {
            // Object header like "#0", "#1", etc. (not verb program like "#0:0")
            if objects_start.is_none() {
                objects_start = Some(i);
                println!("Objects start at line {}: '{}'", i+1, line);
            }
        }
        if line.contains(":") && line.starts_with("#") {
            // Verb program header like "#0:0", "#1:1", etc.
            verb_programs_start = Some(i);
            println!("Verb programs start at line {}: '{}'", i+1, line);
            break;
        }
    }
    
    if let (Some(start), Some(end)) = (objects_start, verb_programs_start) {
        let object_section = lines[start..end].join("\n");
        println!("\nObject section ({} lines):", end - start);
        println!("Content:\n{}", object_section);
        
        // Test parsing this as object_list
        println!("\n=== Testing as object_list ===");
        match LambdaMooDbParser::parse(Rule::object_list, &object_section) {
            Ok(mut pairs) => {
                println!("✅ object_list parsed successfully");
                if let Some(pair) = pairs.next() {
                    println!("Full match length: {}", pair.as_str().len());
                    println!("Input length: {}", object_section.len());
                    
                    let mut object_count = 0;
                    for inner in pair.into_inner() {
                        match inner.as_rule() {
                            Rule::object_count => {
                                println!("  Object count declared: {}", inner.as_str());
                            }
                            Rule::object_def => {
                                object_count += 1;
                                // Extract object ID from object_def
                                for obj_inner in inner.into_inner() {
                                    match obj_inner.as_rule() {
                                        Rule::object_header => {
                                            let header_text = obj_inner.as_str();
                                            println!("  Object {}: {}", object_count, header_text.trim());
                                            break;
                                        }
                                        Rule::recycled_marker => {
                                            println!("  Object {}: recycled", object_count);
                                            break;
                                        }
                                        Rule::object_body => {
                                            println!("  Object {}: has body", object_count);
                                            break;
                                        }
                                        _ => {}
                                    }
                                }
                            }
                            _ => {
                                println!("  Other rule: {:?} = '{}'", inner.as_rule(), inner.as_str().chars().take(20).collect::<String>());
                            }
                        }
                    }
                    println!("Total objects found: {}", object_count);
                }
            }
            Err(e) => println!("❌ object_list failed: {}", e),
        }
        
        // Also test individual object_def parsing
        println!("\n=== Testing individual objects ===");
        let mut current_object = Vec::new();
        let mut object_num = 0;
        
        for line in lines[start..end].iter() {
            if line.starts_with("#") && !current_object.is_empty() {
                // Start of new object, test the previous one
                let obj_content = current_object.join("\n");
                object_num += 1;
                
                match LambdaMooDbParser::parse(Rule::object_def, &obj_content) {
                    Ok(_) => println!("✅ Object {} parses individually", object_num),
                    Err(e) => println!("❌ Object {} failed individually: {}", object_num, e),
                }
                
                current_object.clear();
            }
            current_object.push(*line);
        }
        
        // Test the last object
        if !current_object.is_empty() {
            let obj_content = current_object.join("\n");
            object_num += 1;
            
            match LambdaMooDbParser::parse(Rule::object_def, &obj_content) {
                Ok(_) => println!("✅ Object {} parses individually", object_num),
                Err(e) => println!("❌ Object {} failed individually: {}", object_num, e),
            }
        }
        
    } else {
        println!("Could not find object section boundaries");
    }
    
    Ok(())
}