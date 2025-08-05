use std::fs;
use pest::Parser;
use echo_core::parser::lambdamoo_db_parser::{LambdaMooDbParser, Rule};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing database parsing up to verb programs...");
    
    let content = fs::read_to_string("examples/Minimal.db")?;
    let lines: Vec<&str> = content.lines().collect();
    
    // Create database content up to but not including line 75 (#0:0)
    let partial_content = lines[0..74].join("\n") + "\n";
    
    println!("Testing database up to line 74 ({} chars, {} lines)", 
        partial_content.len(), partial_content.lines().count());
    
    // Test database parsing without verb programs section
    match LambdaMooDbParser::parse(Rule::database, &partial_content) {
        Ok(mut pairs) => {
            println!("✅ Database parsed successfully up to line 74");
            
            if let Some(db_pair) = pairs.next() {
                println!("Analyzing parsed structure:");
                for inner in db_pair.into_inner() {
                    match inner.as_rule() {
                        Rule::object_list => {
                            println!("  Found object_list section");
                            let mut obj_count = 0;
                            for obj_inner in inner.into_inner() {
                                match obj_inner.as_rule() {
                                    Rule::object_def => {
                                        obj_count += 1;
                                        println!("    Object {}", obj_count);
                                    }
                                    Rule::object_count => {
                                        println!("    Object count: {}", obj_inner.as_str());
                                    }
                                    _ => {
                                        println!("    Other in object_list: {:?}", obj_inner.as_rule());
                                    }
                                }
                            }
                            println!("  Total objects found: {}", obj_count);
                        }
                        _ => {
                            println!("  Database section: {:?}", inner.as_rule());
                        }
                    }
                }
            }
        }
        Err(e) => {
            println!("❌ Database parsing failed at: {}", e);
            
            // Let's try parsing just the object_list section directly
            println!("\n=== Testing object_list section directly ===");
            
            // Find the object section (after player list)
            let mut object_start = None;
            for (i, line) in lines.iter().enumerate() {
                if line.starts_with("#0") && !line.contains(":") {
                    object_start = Some(i);
                    break;
                }
            }
            
            if let Some(start) = object_start {
                let object_section = lines[start..74].join("\n") + "\n";
                println!("Object section from line {} to 74: {} chars", start, object_section.len());
                
                match LambdaMooDbParser::parse(Rule::object_list, &object_section) {
                    Ok(mut pairs) => {
                        if let Some(pair) = pairs.next() {
                            println!("✅ object_list parsed directly");
                            
                            let mut obj_count = 0;
                            for inner in pair.into_inner() {
                                match inner.as_rule() {
                                    Rule::object_def => {
                                        obj_count += 1;
                                        println!("  Object {} found", obj_count);
                                    }
                                    _ => {}
                                }
                            }
                            println!("Total objects: {}", obj_count);
                        }
                    }
                    Err(e) => println!("❌ object_list direct parsing failed: {}", e),
                }
            }
        }
    }
    
    Ok(())
}