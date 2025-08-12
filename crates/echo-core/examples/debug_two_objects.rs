use std::fs;
use pest::Parser;
use echo_core::parser::lambdamoo_db_parser::{LambdaMooDbParser, Rule};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing parsing of first two objects...");
    
    let content = fs::read_to_string("examples/LambdaCore-latest.db")?;
    let lines: Vec<&str> = content.lines().collect();
    
    // Extract just the first two objects (lines 10-1006)
    let obj0_start = 10;  // 0-indexed, object #0 starts at line 11
    let obj2_start = 1006; // 0-indexed, object #2 starts at line 1007
    
    let two_objects_section: Vec<&str> = lines[obj0_start..obj2_start].to_vec();
    let two_objects_content = two_objects_section.join("\n") + "\n";
    
    println!("Extracted {} lines (objects #0 and #1)", two_objects_section.len());
    
    // Try to parse as object_list
    match LambdaMooDbParser::parse(Rule::object_list, &two_objects_content) {
        Ok(parsed) => {
            println!("✅ Two objects section parses successfully");
            
            let mut object_count = 0;
            for pair in parsed {
                println!("Parsing object_list rule...");
                for inner in pair.into_inner() {
                    match inner.as_rule() {
                        Rule::object_def => {
                            object_count += 1;
                            println!("Found object_def #{}", object_count);
                        }
                        _ => {
                            println!("Found other rule: {:?}", inner.as_rule());
                        }
                    }
                }
            }
            println!("Total object definitions found: {}", object_count);
        }
        Err(e) => {
            println!("❌ Two objects parsing failed: {}", e);
        }
    }
    
    Ok(())
}