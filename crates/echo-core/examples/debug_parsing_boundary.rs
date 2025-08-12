use std::fs;
use pest::Parser;
use echo_core::parser::lambdamoo_db_parser::{LambdaMooDbParser, Rule};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Debugging exactly where parsing stops...");
    
    let content = fs::read_to_string("examples/LambdaCore-latest.db")?;
    let lines: Vec<&str> = content.lines().collect();
    
    // Test parsing property_values section directly
    let prop_values_start = 230; // Line 231 has "123" (property values count)
    let obj1_start = 820; // Line 821 has "#1"
    
    // Extract just the property values section
    let prop_values_lines: Vec<&str> = lines[prop_values_start..obj1_start].to_vec();
    let prop_values_content = prop_values_lines.join("\n") + "\n";
    
    println!("Property values section: {} lines (from {} to {})", prop_values_lines.len(), prop_values_start + 1, obj1_start);
    println!("First few lines:");
    for (i, line) in prop_values_lines.iter().take(5).enumerate() {
        println!("  {}: '{}'", prop_values_start + i + 1, line);
    }
    println!("Last few lines:");
    for (i, line) in prop_values_lines.iter().rev().take(5).rev().enumerate() {
        let line_num = obj1_start - 5 + i;
        println!("  {}: '{}'", line_num + 1, line);
    }
    
    // Test parsing property_values rule directly
    match LambdaMooDbParser::parse(Rule::property_values, &prop_values_content) {
        Ok(parsed) => {
            println!("✅ Property values section parses successfully");
            
            let mut propval_count = 0;
            for pair in parsed {
                println!("property_values rule matched");
                for inner in pair.into_inner() {
                    match inner.as_rule() {
                        Rule::propval_count => {
                            println!("  Found propval_count: {}", inner.as_str());
                        }
                        Rule::propval => {
                            propval_count += 1;
                            if propval_count <= 5 || propval_count > 30 {
                                println!("  Found propval #{}", propval_count);
                            } else if propval_count == 6 {
                                println!("  ... (skipping propvals 6-30)");
                            }
                        }
                        _ => {
                            println!("  Found other rule: {:?}", inner.as_rule());
                        }
                    }
                }
            }
            println!("Total propvals found: {} (expected: 123)", propval_count);
        }
        Err(e) => {
            println!("❌ Property values parsing failed: {}", e);
        }
    }
    
    // Now test parsing object #0 by itself
    println!("\n--- Testing Object #0 Parsing ---");
    let obj0_start = 10; // Line 11 has "#0"
    let obj0_lines: Vec<&str> = lines[obj0_start..obj1_start].to_vec();
    let obj0_content = obj0_lines.join("\n") + "\n";
    
    println!("Object #0: {} lines", obj0_lines.len());
    
    match LambdaMooDbParser::parse(Rule::object_def, &obj0_content) {
        Ok(parsed) => {
            println!("✅ Object #0 parses successfully as object_def");
        }
        Err(e) => {
            println!("❌ Object #0 parsing failed: {}", e);
        }
    }
    
    Ok(())
}