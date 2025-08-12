use std::fs;
use pest::Parser;
use echo_core::parser::lambdamoo_db_parser::{LambdaMooDbParser, Rule};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing parsing of a single property value...");
    
    let content = fs::read_to_string("examples/LambdaCore-latest.db")?;
    let lines: Vec<&str> = content.lines().collect();
    
    // Extract just one property value (4 lines starting from line 232)
    let propval_start = 231; // 0-indexed, line 232 in file
    let single_propval_lines: Vec<&str> = lines[propval_start..propval_start + 4].to_vec();
    let single_propval_content = single_propval_lines.join("\n") + "\n";
    
    println!("Single property value content:");
    for (i, line) in single_propval_lines.iter().enumerate() {
        println!("  {}: '{}'", propval_start + i + 1, line);
    }
    
    // Try to parse as single propval
    match LambdaMooDbParser::parse(Rule::propval, &single_propval_content) {
        Ok(parsed) => {
            println!("✅ Single property value parses successfully");
            for pair in parsed {
                println!("propval rule matched, span: {}", pair.as_str().len());
            }
        }
        Err(e) => {
            println!("❌ Single property value parsing failed: {}", e);
        }
    }
    
    // Now try parsing just the value part (first 2 lines)
    let value_lines: Vec<&str> = lines[propval_start..propval_start + 2].to_vec();
    let value_content = value_lines.join("\n") + "\n";
    
    println!("\nValue content:");
    for (i, line) in value_lines.iter().enumerate() {
        println!("  {}: '{}'", propval_start + i + 1, line);
    }
    
    match LambdaMooDbParser::parse(Rule::value, &value_content) {
        Ok(parsed) => {
            println!("✅ Value parses successfully");
            for pair in parsed {
                println!("value rule matched, span: {}", pair.as_str().len());
            }
        }
        Err(e) => {
            println!("❌ Value parsing failed: {}", e);
        }
    }
    
    // Test prop_value_type rule directly
    let type_content = "1";
    println!("\nTesting prop_value_type with: '{}'", type_content);
    match LambdaMooDbParser::parse(Rule::prop_value_type, type_content) {
        Ok(parsed) => {
            println!("✅ prop_value_type parses successfully");
        }
        Err(e) => {
            println!("❌ prop_value_type parsing failed: {}", e);
        }
    }
    
    // Test prop_value_content rule directly
    let content = "4";
    println!("\nTesting prop_value_content with: '{}'", content);
    match LambdaMooDbParser::parse(Rule::prop_value_content, content) {
        Ok(parsed) => {
            println!("✅ prop_value_content parses successfully");
        }
        Err(e) => {
            println!("❌ prop_value_content parsing failed: {}", e);
        }
    }
    
    Ok(())
}