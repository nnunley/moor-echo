use pest::Parser;
use echo_core::parser::lambdamoo_db_parser::{LambdaMooDbParser, Rule};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing verb_def newline requirements...");
    
    // Test 1: Single verb with trailing newline
    println!("=== Test 1: Single verb with trailing newline ===");
    let single_verb_with_newline = r#"do_start_script
3
173
-1
"#;
    
    match LambdaMooDbParser::parse(Rule::verb_def, single_verb_with_newline) {
        Ok(_) => println!("✅ Single verb with newline parses OK"),
        Err(e) => println!("❌ Single verb with newline failed: {}", e),
    }
    
    // Test 2: Single verb without trailing newline
    println!("\n=== Test 2: Single verb without trailing newline ===");
    let single_verb_no_newline = r#"do_start_script
3
173
-1"#;
    
    match LambdaMooDbParser::parse(Rule::verb_def, single_verb_no_newline) {
        Ok(_) => println!("✅ Single verb without newline parses OK"),
        Err(e) => println!("❌ Single verb without newline failed: {}", e),
    }
    
    // Test 3: Two verbs properly formatted
    println!("\n=== Test 3: Two verbs properly formatted ===");
    let two_verbs = r#"2
do_start_script
3
173
-1
do_login_command
3
173
-1
"#;
    
    match LambdaMooDbParser::parse(Rule::verb_definitions, two_verbs) {
        Ok(mut pairs) => {
            println!("✅ Two verbs parsed");
            if let Some(pair) = pairs.next() {
                println!("Full match length: {}", pair.as_str().len());
                println!("Input length: {}", two_verbs.len());
                
                let mut verb_count = 0;
                for inner in pair.into_inner() {
                    match inner.as_rule() {
                        Rule::verb_count => println!("  Verb count: {}", inner.as_str()),
                        Rule::verb_def => {
                            verb_count += 1;
                            println!("  Verb {}: {}", verb_count, inner.as_str().lines().next().unwrap_or(""));
                        }
                        _ => {}
                    }
                }
                println!("  Total verbs parsed: {}", verb_count);
            }
        }
        Err(e) => println!("❌ Two verbs failed: {}", e),
    }
    
    // Test 4: The exact problem - two verbs without final newline
    println!("\n=== Test 4: Two verbs without final newline (the problem) ===");
    let two_verbs_no_final_newline = r#"2
do_start_script
3
173
-1
do_login_command
3
173
-1"#;
    
    match LambdaMooDbParser::parse(Rule::verb_definitions, two_verbs_no_final_newline) {
        Ok(mut pairs) => {
            println!("✅ Two verbs without final newline parsed");
            if let Some(pair) = pairs.next() {
                println!("Full match length: {}", pair.as_str().len());
                println!("Input length: {}", two_verbs_no_final_newline.len());
                
                let mut verb_count = 0;
                for inner in pair.into_inner() {
                    match inner.as_rule() {
                        Rule::verb_count => println!("  Verb count: {}", inner.as_str()),
                        Rule::verb_def => {
                            verb_count += 1;
                            println!("  Verb {}: {}", verb_count, inner.as_str().lines().next().unwrap_or(""));
                        }
                        _ => {}
                    }
                }
                println!("  Total verbs parsed: {}", verb_count);
            }
        }
        Err(e) => println!("❌ Two verbs without final newline failed: {}", e),
    }
    
    Ok(())
}