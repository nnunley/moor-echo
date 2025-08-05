use std::fs;
use pest::Parser;
use echo_core::parser::lambdamoo_db_parser::{LambdaMooDbParser, Rule};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing Pest grammar rules...");
    
    // Test 1: Simple object header
    println!("\n=== Test 1: Object Header ===");
    let object_header = "#0\n";
    match LambdaMooDbParser::parse(Rule::object_header, object_header) {
        Ok(pairs) => {
            println!("✅ object_header parsed successfully");
            for pair in pairs {
                println!("  Rule: {:?}, Text: '{}'", pair.as_rule(), pair.as_str());
            }
        }
        Err(e) => println!("❌ object_header failed: {}", e),
    }
    
    // Test 2: Object name
    println!("\n=== Test 2: Object Name ===");
    let object_name = "The System Object\n";
    match LambdaMooDbParser::parse(Rule::object_name, object_name) {
        Ok(pairs) => {
            println!("✅ object_name parsed successfully");
            for pair in pairs {
                println!("  Rule: {:?}, Text: '{}'", pair.as_rule(), pair.as_str());
            }
        }
        Err(e) => println!("❌ object_name failed: {}", e),
    }
    
    // Test 3: Test verb definitions specifically
    println!("\n=== Test 3: Verb Definitions ===");
    let verb_defs = r#"1
do_login_command
2
173
-1
"#;
    match LambdaMooDbParser::parse(Rule::verb_definitions, verb_defs) {
        Ok(pairs) => {
            println!("✅ verb_definitions parsed successfully");
            for pair in pairs {
                println!("  Rule: {:?}, Text: '{}'", pair.as_rule(), pair.as_str().chars().take(50).collect::<String>());
                for inner in pair.into_inner() {
                    println!("    Inner Rule: {:?}, Text: '{}'", inner.as_rule(), inner.as_str().chars().take(30).collect::<String>());
                }
            }
        }
        Err(e) => println!("❌ verb_definitions failed: {}", e),
    }
    
    // Test 3b: Test ToastStunt extended verb definitions
    println!("\n=== Test 3b: ToastStunt Extended Verb Definitions ===");
    let toaststunt_verb_defs = r#"1
4
0
27
do_login_command
2
173
-1
"#;
    match LambdaMooDbParser::parse(Rule::verb_definitions_extended, toaststunt_verb_defs) {
        Ok(pairs) => {
            println!("✅ verb_definitions_extended parsed successfully");
            for pair in pairs {
                println!("  Rule: {:?}, Text: '{}'", pair.as_rule(), pair.as_str().chars().take(50).collect::<String>());
                for inner in pair.into_inner() {
                    println!("    Inner Rule: {:?}, Text: '{}'", inner.as_rule(), inner.as_str().chars().take(30).collect::<String>());
                }
            }
        }
        Err(e) => println!("❌ verb_definitions_extended failed: {}", e),
    }
    
    // Test 4: Complete minimal object
    println!("\n=== Test 4: Complete Object Definition ===");
    let complete_object = r#"#0
The System Object
24
2
1
-1
0
0
4
0
1
1
0

0

"#;
    match LambdaMooDbParser::parse(Rule::object_def, complete_object) {
        Ok(pairs) => {
            println!("✅ complete object_def parsed successfully");
            for pair in pairs {
                println!("  Rule: {:?}, Text: '{}'", pair.as_rule(), pair.as_str().chars().take(50).collect::<String>());
                for inner in pair.into_inner() {
                    println!("    Inner Rule: {:?}, Text: '{}'", inner.as_rule(), inner.as_str().chars().take(30).collect::<String>());
                }
            }
        }
        Err(e) => println!("❌ complete object_def failed: {}", e),
    }
    
    // Test 5: Object definition from real file
    println!("\n=== Test 5: Real Object from File ===");
    if let Ok(content) = fs::read_to_string("/tmp/object_0_sample.txt") {
        // Take just the first 20 lines for a simpler test
        let lines: Vec<&str> = content.lines().collect();
        let simplified_object = format!("{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n0\n\n0\n\n", 
            lines[0], lines[1], lines[2], lines[3], lines[4], lines[5], 
            lines[6], lines[7], lines[8], lines[9], lines[10], lines[11]);
        
        println!("Testing with simplified object:");
        println!("{}", simplified_object);
        
        match LambdaMooDbParser::parse(Rule::object_def, &simplified_object) {
            Ok(pairs) => {
                println!("✅ real object_def parsed successfully");
                for pair in pairs {
                    println!("  Rule: {:?}", pair.as_rule());
                    for inner in pair.into_inner() {
                        println!("    Inner Rule: {:?}, Text: '{}'", inner.as_rule(), inner.as_str().chars().take(30).collect::<String>());
                    }
                }
            }
            Err(e) => println!("❌ real object_def failed: {}", e),
        }
    }
    
    // Test 6: Test individual property sections
    println!("\n=== Test 6: Property Definitions ===");
    let prop_defs = "0\n";
    match LambdaMooDbParser::parse(Rule::property_definitions, prop_defs) {
        Ok(_) => println!("✅ property_definitions with 0 count works"),
        Err(e) => println!("❌ property_definitions failed: {}", e),
    }
    
    // Test 7: Test property values
    println!("\n=== Test 7: Property Values ===");
    let prop_vals = "0\n";
    match LambdaMooDbParser::parse(Rule::property_values, prop_vals) {
        Ok(_) => println!("✅ property_values with 0 count works"),
        Err(e) => println!("❌ property_values failed: {}", e),
    }
    
    // Test 8: Test lambdamoo_object_body directly
    println!("\n=== Test 8: LambdaMOO Object Body ===");
    let lambdamoo_body = r#"System Object

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
0
"#;
    match LambdaMooDbParser::parse(Rule::lambdamoo_object_body, lambdamoo_body) {
        Ok(pairs) => {
            println!("✅ lambdamoo_object_body parsed successfully");
            for pair in pairs {
                println!("  Rule: {:?}", pair.as_rule());
                for inner in pair.into_inner() {
                    println!("    Inner Rule: {:?}, Text: '{}'", inner.as_rule(), inner.as_str().chars().take(30).collect::<String>());
                }
            }
        }
        Err(e) => println!("❌ lambdamoo_object_body failed: {}", e),
    }
    
    // Test 9: Test just the problem section
    println!("\n=== Test 9: Problem Section - Empty line after verb definitions ===");
    let problem_section = r#"2
do_start_script
3
173
-1
do_login_command
3
173
-1
0
"#;
    match LambdaMooDbParser::parse(Rule::verb_definitions, problem_section) {
        Ok(pairs) => {
            println!("✅ verb_definitions with following property section parsed successfully");
            for pair in pairs {
                println!("  Rule: {:?}", pair.as_rule());
                for inner in pair.into_inner() {
                    println!("    Inner Rule: {:?}, Text: '{}'", inner.as_rule(), inner.as_str().chars().take(30).collect::<String>());
                }
            }
        }
        Err(e) => println!("❌ verb_definitions with following property section failed: {}", e),
    }
    
    // Test 10: Test exactly the problem sequence: verb definitions + properties  
    println!("\n=== Test 10: Exact Problem Sequence ===");
    let exact_problem = r#"2
do_start_script
3
173
-1
do_login_command
3
173
-1
0

0

"#;
    // Parse the parts separately to see where it breaks
    println!("Testing verb_definitions + property_definitions + property_values sequence");
    
    // First check if the issue is a trailing newline problem
    println!("Length of test string: {}", exact_problem.len());
    println!("Last 10 chars: {:?}", &exact_problem.chars().rev().take(10).collect::<Vec<_>>());
    
    // Try parsing just the sequence "verb_defs + prop_defs + prop_vals" as would appear in lambdamoo_object_body
    let sequence_test = r#"2
do_start_script
3
173
-1
do_login_command
3
173
-1
0
0
"#;
    println!("Testing sequence without extra newlines:");
    
    Ok(())
}