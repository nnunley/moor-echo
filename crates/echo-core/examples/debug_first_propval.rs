use std::fs;
use pest::Parser;
use echo_core::parser::lambdamoo_db_parser::{LambdaMooDbParser, Rule};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing first property value parsing...");
    
    let content = fs::read_to_string("examples/LambdaCore-latest.db")?;
    let lines: Vec<&str> = content.lines().collect();
    
    // The first property value should be at lines 232-235 (1-indexed)
    // But let's find the right structure by looking at the lines
    println!("Lines around first property value:");
    for i in 230..240 {
        println!("  {}: '{}'", i + 1, lines[i]);
    }
    
    // Test parsing the first property value (should be type 1, content 4, owner 2, perms 5)
    let propval_lines = vec![lines[231], lines[232], lines[233], lines[234]]; // 0-indexed
    let propval_content = propval_lines.join("\n");
    
    println!("\nFirst property value content:");
    for (i, line) in propval_lines.iter().enumerate() {
        println!("  Line {}: '{}'", i + 232, line);
    }
    
    // Test if it parses as a propval
    match LambdaMooDbParser::parse(Rule::propval, &propval_content) {
        Ok(parsed) => {
            println!("✅ First property value parses as propval");
            for pair in parsed {
                println!("Propval span: {} chars", pair.as_str().len());
            }
        }
        Err(e) => {
            println!("❌ First property value parsing failed: {}", e);
        }
    }
    
    // Test parsing just the value part
    let value_lines = vec![lines[231], lines[232]]; // type and content
    let value_content = value_lines.join("\n") + "\n";
    
    println!("\nTesting just the value part:");
    for (i, line) in value_lines.iter().enumerate() {
        println!("  Line {}: '{}'", i + 232, line);
    }
    
    match LambdaMooDbParser::parse(Rule::value, &value_content) {
        Ok(parsed) => {
            println!("✅ Value parses successfully");
        }
        Err(e) => {
            println!("❌ Value parsing failed: {}", e);
        }
    }
    
    Ok(())
}