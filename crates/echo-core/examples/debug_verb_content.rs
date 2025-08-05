use pest::Parser;
use echo_core::parser::lambdamoo_db_parser::{LambdaMooDbParser, Rule};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Debugging verb content parsing...");
    
    let verb_section = r#"2
do_start_script
3
173
-1
do_login_command
3
173
-1"#;
    
    println!("Verb section content:");
    for (i, line) in verb_section.lines().enumerate() {
        println!("  Line {}: '{}'", i+1, line);
    }
    println!("Total lines: {}", verb_section.lines().count());
    println!("Total length: {}", verb_section.len());
    
    match LambdaMooDbParser::parse(Rule::verb_definitions, verb_section) {
        Ok(mut pairs) => {
            println!("\n✅ verb_definitions parsed");
            
            if let Some(pair) = pairs.next() {
                println!("Full match: '{}'", pair.as_str());
                println!("Match length: {}", pair.as_str().len());
                
                // Look at inner rules
                for inner in pair.into_inner() {
                    println!("  Inner rule: {:?}", inner.as_rule());
                    match inner.as_rule() {
                        Rule::verb_count => {
                            println!("    Verb count: {}", inner.as_str());
                        }
                        Rule::verb_def => {
                            println!("    Verb def: '{}'", inner.as_str().replace('\n', "\\n"));
                            // Look at verb def inner parts
                            for verb_part in inner.into_inner() {
                                println!("      Part: {:?} = '{}'", verb_part.as_rule(), verb_part.as_str());
                            }
                        }
                        _ => {
                            println!("    Other: {}", inner.as_str());
                        }
                    }
                }
            }
        }
        Err(e) => println!("❌ verb_definitions failed: {}", e),
    }
    
    // Test individual verb parsing
    println!("\n=== Testing individual verb parsing ===");
    let single_verb = r#"do_start_script
3
173
-1"#;
    
    match LambdaMooDbParser::parse(Rule::verb_def, single_verb) {
        Ok(_) => println!("✅ Single verb parses OK"),
        Err(e) => println!("❌ Single verb failed: {}", e),
    }
    
    let second_verb = r#"do_login_command
3
173
-1"#;
    
    match LambdaMooDbParser::parse(Rule::verb_def, second_verb) {
        Ok(_) => println!("✅ Second verb parses OK"),
        Err(e) => println!("❌ Second verb failed: {}", e),
    }
    
    Ok(())
}