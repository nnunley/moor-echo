use std::fs;
use pest::Parser;
use echo_core::parser::lambdamoo_db_parser::{LambdaMooDbParser, Rule};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing sequence parsing to find the exact issue...");
    
    // Build object body step by step to see where it breaks
    
    // Step 1: Header fields only
    let step1 = r#"System Object

16
3
-1
-1
-1
1
-1
2
"#;
    
    println!("=== Step 1: Up to object_sibling ===");
    // Can't test this directly as it's not a complete rule, but we can test the sequence
    
    // Step 2: Add verb definitions
    let step2 = r#"System Object

16
3
-1
-1
-1
1
-1
2
2
do_start_script
3
173
-1
do_login_command
3
173
-1"#;
    
    println!("=== Step 2: Add verb definitions ===");
    // Test if this parses as beginning of lambdamoo_object_body
    
    // Step 3: Add property definitions
    let step3 = r#"System Object

16
3
-1
-1
-1
1
-1
2
2
do_start_script
3
173
-1
do_login_command
3
173
-1
0"#;
    
    println!("=== Step 3: Add property definitions count ===");
    match LambdaMooDbParser::parse(Rule::lambdamoo_object_body, step3) {
        Ok(_) => println!("✅ With property_definitions count: SUCCESS"),
        Err(e) => println!("❌ With property_definitions count failed: {}", e),
    }
    
    // Step 4: Add property values (the failing part)
    let step4 = r#"System Object

16
3
-1
-1
-1
1
-1
2
2
do_start_script
3
173
-1
do_login_command
3
173
-1
0
0"#;
    
    println!("=== Step 4: Add property values count ===");
    match LambdaMooDbParser::parse(Rule::lambdamoo_object_body, step4) {
        Ok(_) => println!("✅ With property_values count: SUCCESS"),
        Err(e) => println!("❌ With property_values count failed: {}", e),
    }
    
    // Let me test the exact problematic part: "verb_definitions ~ property_definitions ~ property_values"
    println!("\n=== Testing just the three end sections ===");
    let three_sections = r#"2
do_start_script
3
173
-1
do_login_command
3
173
-1
0
0"#;
    
    println!("Content: '{}'", three_sections);
    
    // This should work according to the grammar: verb_definitions ~ property_definitions ~ property_values
    // But we can't test this directly. Let me try a different approach.
    
    // Test if the problem is with newlines in property_values
    println!("\n=== Testing property_values parsing issue ===");
    
    // The individual property_values rule works with "0\n", so let's see what happens in sequence
    let prop_sequence = r#"0
0"#;
    println!("Testing property sequence: '{}'", prop_sequence);
    
    // Let me try one more approach - testing if the issue is that property_definitions consumes the newline
    println!("\n=== Final diagnosis attempt ===");
    let diagnosis = r#"0
0
"#;
    
    match LambdaMooDbParser::parse(Rule::property_definitions, "0\n") {
        Ok(pairs) => {
            println!("✅ property_definitions('0\\n') works");
            for pair in pairs {
                println!("  Consumed: '{}'", pair.as_str());
            }
        }
        Err(e) => println!("❌ property_definitions('0\\n') failed: {}", e),
    }
    
    match LambdaMooDbParser::parse(Rule::property_values, "0\n") {
        Ok(pairs) => {
            println!("✅ property_values('0\\n') works");
            for pair in pairs {
                println!("  Consumed: '{}'", pair.as_str());
            }
        }
        Err(e) => println!("❌ property_values('0\\n') failed: {}", e),
    }
    
    Ok(())
}