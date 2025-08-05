use std::fs;
use pest::Parser;
use echo_core::parser::lambdamoo_db_parser::{LambdaMooDbParser, Rule};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Debugging LambdaCore 'invalid digit found in string' error...");
    
    let content = fs::read_to_string("examples/LambdaCore-latest.db")?;
    println!("LambdaCore database: {} chars, {} lines", content.len(), content.lines().count());
    
    // Look at the first 30 lines to understand the structure
    println!("\n=== First 30 lines of LambdaCore ===");
    for (i, line) in content.lines().take(30).enumerate() {
        println!("{:3}: '{}'", i+1, line);
    }
    
    // Try parsing just the header and intro sections
    println!("\n=== Testing partial parsing ===");
    let lines: Vec<&str> = content.lines().collect();
    
    // Test header only
    let header_content = format!("{}\n", lines[0]);
    match LambdaMooDbParser::parse(Rule::header, &header_content) {
        Ok(_) => println!("✅ Header parses successfully"),
        Err(e) => println!("❌ Header parsing failed: {}", e),
    }
    
    // Test intro block (lines 2-5)
    let intro_content = lines[1..5].join("\n") + "\n";
    match LambdaMooDbParser::parse(Rule::intro_block, &intro_content) {
        Ok(_) => println!("✅ Intro block parses successfully"),
        Err(e) => println!("❌ Intro block parsing failed: {}", e),
    }
    
    // Test database up to first 20 lines
    let partial_db = lines[0..20].join("\n") + "\n";
    match LambdaMooDbParser::parse(Rule::database, &partial_db) {
        Ok(_) => println!("✅ First 20 lines parse successfully"),
        Err(e) => println!("❌ First 20 lines parsing failed: {}", e),
    }
    
    // Find where parsing starts to fail by testing incrementally
    println!("\n=== Finding failure point ===");
    for test_lines in [50, 100, 200, 500, 1000, 2000, 5000].iter() {
        if *test_lines > lines.len() {
            continue;
        }
        
        let test_content = lines[0..*test_lines].join("\n") + "\n";
        match LambdaMooDbParser::parse(Rule::database, &test_content) {
            Ok(_) => println!("✅ First {} lines parse successfully", test_lines),
            Err(e) => {
                println!("❌ First {} lines parsing failed: {}", test_lines, e);
                
                // Try a smaller range to narrow down the issue
                if *test_lines > 100 {
                    let prev_size = if *test_lines == 200 { 100 } else { test_lines - 100 };
                    println!("   Issue is between lines {} and {}", prev_size, test_lines);
                }
                break;
            }
        }
    }
    
    Ok(())
}