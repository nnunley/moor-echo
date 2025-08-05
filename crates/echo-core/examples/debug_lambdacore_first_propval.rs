use std::fs;
use pest::Parser;
use echo_core::parser::lambdamoo_db_parser::{LambdaMooDbParser, Rule};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Debugging LambdaCore first property value where error likely occurs...");
    
    let content = fs::read_to_string("examples/LambdaCore-latest.db")?;
    let lines: Vec<&str> = content.lines().collect();
    
    // Based on previous analysis, property values start at line 231
    let prop_vals_start = 230; // 0-indexed (line 231)
    
    println!("Property values section starts at line {}", prop_vals_start + 1);
    println!("Property values count: '{}'", lines[prop_vals_start]);
    
    // The first property value should start at line 232
    // Property value format:
    // - value_type (number)
    // - value_content (depends on type)
    // - owner (object id)
    // - perms (number)
    
    println!("\nFirst property value structure:");
    for i in 1..15 {  // Start from line 232 (index 231)
        if prop_vals_start + i < lines.len() {
            let line_num = prop_vals_start + i + 1;
            let line = lines[prop_vals_start + i];
            println!("{:3}: '{}'", line_num, line);
            
            match i {
                1 => {
                    println!("     ^ Should be value_type");
                    match line.parse::<i64>() {
                        Ok(vtype) => {
                            println!("     ✅ Value type: {}", vtype);
                            match vtype {
                                -2 => println!("        TYPE_CLEAR"),
                                -1 => println!("        TYPE_NONE"),
                                0 => println!("        TYPE_STR"),
                                1 => println!("        TYPE_OBJ"),
                                2 => println!("        TYPE_ERR"),
                                3 => println!("        TYPE_INT"),
                                4 => println!("        TYPE_CATCH"),
                                5 => println!("        TYPE_FINALLY"),
                                9 => println!("        TYPE_FLOAT"),
                                10 => println!("        TYPE_LIST"),
                                12 => println!("        TYPE_MAP"),
                                _ => println!("        UNKNOWN TYPE"),
                            }
                        }
                        Err(e) => println!("     ❌ Invalid value type: {}", e),
                    }
                }
                2 => println!("     ^ Should be value_content"),
                3 => {
                    println!("     ^ Should be owner");
                    match line.parse::<i64>() {
                        Ok(owner) => println!("     ✅ Owner: {}", owner),
                        Err(e) => println!("     ❌ Invalid owner: {}", e),
                    }
                }
                4 => {
                    println!("     ^ Should be perms");
                    match line.parse::<i64>() {
                        Ok(perms) => println!("     ✅ Perms: {}", perms),
                        Err(e) => println!("     ❌ Invalid perms: {}", e),
                    }
                }
                _ => println!("     ^ Next property value or other"),
            }
        }
    }
    
    // Try to parse just the first property value
    println!("\nTrying to parse first property value...");
    
    // Assuming the first property value is TYPE_OBJ (type 1) with simple structure
    let first_propval_lines = &lines[prop_vals_start + 1..prop_vals_start + 5]; // Lines 232-235
    let first_propval_content = first_propval_lines.join("\n") + "\n";
    
    println!("First property value content:");
    println!("{}", first_propval_content);
    
    match LambdaMooDbParser::parse(Rule::propval, &first_propval_content) {
        Ok(parsed) => {
            println!("✅ First property value parses successfully");
            for pair in parsed {
                println!("Parsed rule: {:?}", pair.as_rule());
                for inner in pair.into_inner() {
                    println!("  Inner rule: {:?} = '{}'", inner.as_rule(), inner.as_str());
                }
            }
        }
        Err(e) => println!("❌ First property value parsing failed: {}", e),
    }
    
    // Try to parse just the value part
    println!("\nTrying to parse just the value part...");
    let value_lines = &lines[prop_vals_start + 1..prop_vals_start + 3]; // Lines 232-233
    let value_content = value_lines.join("\n") + "\n";
    
    println!("Value content:");
    println!("{}", value_content);
    
    match LambdaMooDbParser::parse(Rule::value, &value_content) {
        Ok(parsed) => {
            println!("✅ Value parses successfully");
            for pair in parsed {
                println!("Parsed rule: {:?}", pair.as_rule());
                for inner in pair.into_inner() {
                    println!("  Inner rule: {:?} = '{}'", inner.as_rule(), inner.as_str());
                }
            }
        }
        Err(e) => println!("❌ Value parsing failed: {}", e),
    }
    
    Ok(())
}