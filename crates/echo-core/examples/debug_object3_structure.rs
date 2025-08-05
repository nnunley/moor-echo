use pest::Parser;
use echo_core::parser::lambdamoo_db_parser::{LambdaMooDbParser, Rule};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Analyzing object #3 structure to understand the extra 0...");
    
    // Let's test different theories about object #3 structure
    
    // Theory 1: Object #3 has 0 verbs, 0 propdefs, 1 propval
    println!("=== Theory 1: Object #3 has 1 property value ===");
    
    // If propval_count is 1, then the last "0" would be a property value
    // A propval needs: value ~ prop_owner ~ newline ~ prop_perms ~ newline
    // So a minimal propval would be: type\ncontent\nowner\nperms
    
    // Let's try to parse just the property values section
    let propval_theory = "1\n0\n0\n0";  // count=1, then a property value
    match LambdaMooDbParser::parse(Rule::property_values, propval_theory) {
        Ok(mut pairs) => {
            if let Some(pair) = pairs.next() {
                println!("✅ propval theory parsed: '{}'", pair.as_str().replace('\n', "\\n"));
                
                // Analyze the structure
                for inner in pair.into_inner() {
                    match inner.as_rule() {
                        Rule::propval_count => println!("  propval_count: '{}'", inner.as_str()),
                        Rule::propval => {
                            println!("  propval found: '{}'", inner.as_str().replace('\n', "\\n"));
                            
                            for propval_inner in inner.into_inner() {
                                match propval_inner.as_rule() {
                                    Rule::value => println!("    value: '{}'", propval_inner.as_str().replace('\n', "\\n")),
                                    Rule::prop_owner => println!("    prop_owner: '{}'", propval_inner.as_str()),
                                    Rule::prop_perms => println!("    prop_perms: '{}'", propval_inner.as_str()),
                                    _ => println!("    other: {:?} = '{}'", propval_inner.as_rule(), propval_inner.as_str()),
                                }
                            }
                        }
                        _ => println!("  other: {:?} = '{}'", inner.as_rule(), inner.as_str()),
                    }
                }
            }
        }
        Err(e) => println!("❌ propval theory failed: {}", e),
    }
    
    // Theory 2: Let's check what value type 0 represents
    println!("\n=== Theory 2: Check value type 0 ===");
    let value_type_0 = "0\n";  // type 0 with empty content
    match LambdaMooDbParser::parse(Rule::value, value_type_0) {
        Ok(mut pairs) => {
            if let Some(pair) = pairs.next() {
                println!("✅ value type 0 parsed: '{}'", pair.as_str().replace('\n', "\\n"));
            }
        }
        Err(e) => println!("❌ value type 0 failed: {}", e),
    }
    
    // Theory 3: Maybe object #3 has a different structure - let's parse it step by step
    println!("\n=== Theory 3: Parse object #3 step by step ===");
    
    // Try parsing as if it has 1 property value with value type 0
    let object3_alt_structure = "#3\nWizard\n\n7\n3\n2\n-1\n-1\n1\n-1\n-1\n0\n0\n1\n0\n\n0\n0";
    
    println!("Testing alternative object #3 structure...");
    match LambdaMooDbParser::parse(Rule::object_def, object3_alt_structure) {
        Ok(mut pairs) => {
            if let Some(pair) = pairs.next() {
                println!("✅ Alternative structure parsed");
                println!("Consumed: {} chars", pair.as_str().len());
            }
        }
        Err(e) => println!("❌ Alternative structure failed: {}", e),
    }
    
    Ok(())
}