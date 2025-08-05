use std::fs;
use pest::Parser;
use echo_core::parser::lambdamoo_db_parser::{LambdaMooDbParser, Rule};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing the actual object_list section that gets parsed in full database context...");
    
    let content = fs::read_to_string("examples/Minimal.db")?;
    
    // Parse the full database and extract the exact object_list section
    match LambdaMooDbParser::parse(Rule::database, &content) {
        Ok(mut pairs) => {
            if let Some(db_pair) = pairs.next() {
                println!("✅ Full database parsed successfully");
                
                // Find the object_list section
                for inner in db_pair.into_inner() {
                    if let Rule::object_list = inner.as_rule() {
                        println!("\n=== Found object_list section ===");
                        println!("Content: '{}'", inner.as_str().chars().take(100).collect::<String>());
                        println!("Length: {} chars", inner.as_str().len());
                        println!("Lines: {}", inner.as_str().lines().count());
                        
                        // Test parsing this exact content in isolation first
                        println!("\n=== Testing this content in isolation ===");
                        let object_list_content = inner.as_str();
                        
                        // Check inner items
                        let inner_items: Vec<_> = inner.into_inner().collect();
                        println!("Inner items: {}", inner_items.len());
                        
                        for (i, item) in inner_items.iter().enumerate() {
                            match item.as_rule() {
                                Rule::object_count => {
                                    println!("  Item {}: object_count = '{}'", i, item.as_str());
                                }
                                Rule::object_def => {
                                    println!("  Item {}: object_def ({}  chars) = '{}'", 
                                        i, item.as_str().len(), 
                                        item.as_str().lines().next().unwrap_or(""));
                                }
                                _ => {
                                    println!("  Item {}: {:?} = '{}'", i, item.as_rule(), 
                                        item.as_str().chars().take(20).collect::<String>());
                                }
                            }
                        }
                        
                        match LambdaMooDbParser::parse(Rule::object_list, object_list_content) {
                            Ok(mut isolated_pairs) => {
                                if let Some(isolated_pair) = isolated_pairs.next() {
                                    println!("✅ Isolated parsing successful");
                                    
                                    let isolated_items: Vec<_> = isolated_pair.into_inner().collect();
                                    println!("Isolated inner items: {}", isolated_items.len());
                                    
                                    for item in isolated_items.iter() {
                                        match item.as_rule() {
                                            Rule::object_count => println!("  Isolated: object_count = '{}'", item.as_str()),
                                            Rule::object_def => println!("  Isolated: object_def found"),
                                            _ => println!("  Isolated: {:?}", item.as_rule()),
                                        }
                                    }
                                }
                            }
                            Err(e) => println!("❌ Isolated parsing failed: {}", e),
                        }
                        
                        break;
                    }
                }
            }
        }
        Err(e) => println!("❌ Full database parsing failed: {}", e),
    }
    
    Ok(())
}