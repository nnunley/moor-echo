use std::fs;
use pest::Parser;
use echo_core::parser::lambdamoo_db_parser::{LambdaMooDbParser, Rule};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing partial database parsing (without problematic sections)...");
    
    let content = fs::read_to_string("/tmp/minimal_partial.db")?;
    println!("Partial database: {} chars, {} lines", content.len(), content.lines().count());
    
    // Test full database parsing
    match LambdaMooDbParser::parse(Rule::database, &content) {
        Ok(mut pairs) => {
            if let Some(db_pair) = pairs.next() {
                println!("✅ Partial database parsed successfully");
                
                // Find the object_list section
                for inner in db_pair.into_inner() {
                    if let Rule::object_list = inner.as_rule() {
                        println!("\n=== Found object_list section ===");
                        println!("Content length: {} chars", inner.as_str().len());
                        println!("Content lines: {}", inner.as_str().lines().count());
                        
                        // Check inner items
                        let inner_items: Vec<_> = inner.into_inner().collect();
                        println!("Inner items: {}", inner_items.len());
                        
                        for (i, item) in inner_items.iter().enumerate() {
                            match item.as_rule() {
                                Rule::object_count => {
                                    println!("  Item {}: object_count = '{}'", i, item.as_str());
                                }
                                Rule::object_def => {
                                    println!("  Item {}: object_def ({} chars) = '{}'", 
                                        i, item.as_str().len(), 
                                        item.as_str().lines().next().unwrap_or(""));
                                }
                                _ => {
                                    println!("  Item {}: {:?} = '{}'", i, item.as_rule(), 
                                        item.as_str().chars().take(20).collect::<String>());
                                }
                            }
                        }
                        
                        if inner_items.len() > 0 {
                            println!("\n🎉 SUCCESS: object_list contains {} items!", inner_items.len());
                        }
                        
                        break;
                    }
                }
            }
        }
        Err(e) => println!("❌ Partial database parsing failed: {}", e),
    }
    
    Ok(())
}