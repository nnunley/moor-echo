use pest::Parser;
use echo_core::parser::lambdamoo_db_parser::{LambdaMooDbParser, Rule};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing suspended_tasks_section parsing...");
    
    // Test the exact content from the database
    let suspended_content = "0 suspended tasks\n";
    
    println!("Testing: '{}'", suspended_content.replace('\n', "\\n"));
    
    match LambdaMooDbParser::parse(Rule::suspended_tasks_section, suspended_content) {
        Ok(mut pairs) => {
            if let Some(pair) = pairs.next() {
                println!("✅ suspended_tasks_section parsed successfully");
                println!("Consumed: '{}'", pair.as_str().replace('\n', "\\n"));
                
                // Check inner structure
                for inner in pair.into_inner() {
                    match inner.as_rule() {
                        Rule::suspended_count => println!("  suspended_count: '{}'", inner.as_str()),
                        Rule::suspended_task => println!("  suspended_task found"),
                        _ => println!("  other: {:?} = '{}'", inner.as_rule(), inner.as_str()),
                    }
                }
            }
        }
        Err(e) => println!("❌ suspended_tasks_section failed: {}", e),
    }
    
    // Test individual components
    println!("\n=== Testing components ===");
    
    // Test suspended_count
    match LambdaMooDbParser::parse(Rule::suspended_count, "0") {
        Ok(_) => println!("✅ suspended_count '0' parsed"),
        Err(e) => println!("❌ suspended_count failed: {}", e),
    }
    
    // Test the pattern manually
    let pattern_test = "0 suspended tasks";
    println!("\nTesting pattern without newline: '{}'", pattern_test);
    match LambdaMooDbParser::parse(Rule::suspended_tasks_section, pattern_test) {
        Ok(_) => println!("✅ Pattern without newline worked"),
        Err(e) => println!("❌ Pattern without newline failed: {}", e),
    }
    
    Ok(())
}