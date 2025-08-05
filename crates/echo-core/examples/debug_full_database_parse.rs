use std::fs;
use pest::Parser;
use echo_core::parser::lambdamoo_db_parser::{LambdaMooDbParser, Rule};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing full database parsing vs isolated object_list...");
    
    let content = fs::read_to_string("examples/Minimal.db")?;
    
    // Test 1: Full database parsing
    println!("=== Test 1: Full database parsing ===");
    match LambdaMooDbParser::parse(Rule::database, &content) {
        Ok(_) => println!("✅ Full database parses successfully"),
        Err(e) => {
            println!("❌ Full database parsing failed: {}", e);
            println!("This tells us where the parsing stops in context");
        }
    }
    
    // Test 2: Parse just the content up to the object section
    println!("\n=== Test 2: Parse up to object section ===");
    let lines: Vec<&str> = content.lines().collect();
    
    // Find where objects start
    let mut objects_start = None;
    for (i, line) in lines.iter().enumerate() {
        if line.starts_with("#") && !line.contains(":") {
            objects_start = Some(i);
            break;
        }
    }
    
    if let Some(start) = objects_start {
        let pre_objects = lines[0..start].join("\n") + "\n";
        println!("Content before objects ({} lines):", start);
        
        // Try to parse the database structure up to but not including objects
        // We can't easily test this with the current grammar structure, but let's see 
        // what happens if we include some objects
        
        // Test 3: Parse database with just first object
        println!("\n=== Test 3: Database with first object only ===");
        
        // Find where second object starts
        let mut second_object = None;
        for (i, line) in lines[start+1..].iter().enumerate() {
            if line.starts_with("#") && !line.contains(":") {
                second_object = Some(start + 1 + i);
                break;
            }
        }
        
        if let Some(second_start) = second_object {
            let partial_content = lines[0..second_start].join("\n") + "\n#0:0\ncallers() && raise(E_PERM);\nreturn eval(@args);\n.\n#0:1\nreturn #3;\n.\n0 clocks\n0 queued tasks\n0 suspended tasks\n";
            
            match LambdaMooDbParser::parse(Rule::database, &partial_content) {
                Ok(mut pairs) => {
                    println!("✅ Database with first object parses successfully");
                    
                    // Analyze the parsed content
                    if let Some(db_pair) = pairs.next() {
                        println!("Analyzing database structure:");
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
                                    println!("  Total objects in database parse: {}", obj_count);
                                }
                                _ => {
                                    println!("  Database section: {:?}", inner.as_rule());
                                }
                            }
                        }
                    }
                }
                Err(e) => println!("❌ Database with first object failed: {}", e),
            }
        }
    }
    
    Ok(())
}