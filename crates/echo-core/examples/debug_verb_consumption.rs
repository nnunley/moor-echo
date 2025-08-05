use std::fs;
use pest::Parser;
use echo_core::parser::lambdamoo_db_parser::{LambdaMooDbParser, Rule};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing verb definitions consumption...");
    
    // Get the verb section from object #0
    let content = fs::read_to_string("examples/Minimal.db")?;
    let lines: Vec<&str> = content.lines().collect();
    
    // Object #0 verb section: line 18 is verb count (2), lines 19-26 are verbs  
    let verb_section = &lines[17..27].join("\n"); // Lines with verb definitions (0-indexed)
    
    println!("Verb section content:");
    for (i, line) in lines[17..27].iter().enumerate() {
        println!("  Line {}: '{}'", i+18, line);
    }
    
    // Test verb_definitions parsing
    println!("\n=== Testing verb_definitions consumption ===");
    match LambdaMooDbParser::parse(Rule::verb_definitions, verb_section) {
        Ok(mut pairs) => {
            if let Some(pair) = pairs.next() {
                println!("✅ verb_definitions consumed: {} chars", pair.as_str().len());
                println!("Input length: {}", verb_section.len());
                
                if pair.as_str().len() != verb_section.len() {
                    println!("🚨 MISMATCH: verb_definitions didn't consume all input!");
                } else {
                    println!("✅ verb_definitions consumed exactly the right amount");
                }
                
                let consumed = pair.as_str();
                println!("Consumed content ends with: '{}'", 
                    consumed.lines().last().unwrap_or(""));
            }
        }
        Err(e) => println!("❌ verb_definitions failed: {}", e),
    }
    
    // Test what happens when we give verb_definitions extra content
    println!("\n=== Testing verb_definitions with extra content ===");
    let verb_with_extra = format!("{}\n0\n0\n#1", verb_section);
    println!("Testing with extra content ({}  chars):", verb_with_extra.len());
    
    match LambdaMooDbParser::parse(Rule::verb_definitions, &verb_with_extra) {
        Ok(mut pairs) => {
            if let Some(pair) = pairs.next() {
                println!("✅ verb_definitions consumed: {} chars out of {}", 
                    pair.as_str().len(), verb_with_extra.len());
                
                if pair.as_str().len() > verb_section.len() {
                    println!("🚨 PROBLEM: verb_definitions consumed extra content!");
                    println!("Expected to consume: {}", verb_section.len());
                    println!("Actually consumed: {}", pair.as_str().len());
                    
                    let consumed = pair.as_str();
                    println!("Extra content consumed:");
                    let extra_lines: Vec<&str> = consumed.lines().skip(verb_section.lines().count()).collect();
                    for line in extra_lines {
                        println!("  '{}'", line);
                    }
                } else {
                    println!("✅ verb_definitions consumed correct amount");
                }
            }
        }
        Err(e) => println!("❌ verb_definitions with extra failed: {}", e),
    }
    
    Ok(())
}