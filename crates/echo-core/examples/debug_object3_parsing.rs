use pest::Parser;
use echo_core::parser::lambdamoo_db_parser::{LambdaMooDbParser, Rule};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing object #3 parsing specifically...");
    
    // Object #3 content from the database
    let object3_content = "#3\nWizard\n\n7\n3\n2\n-1\n-1\n1\n-1\n-1\n0\n0\n0";
    
    println!("Object #3 content:");
    for (i, line) in object3_content.lines().enumerate() {
        println!("  Line {}: '{}'", i+1, line);
    }
    
    // Test parsing object #3
    println!("\n=== Testing object #3 parsing ===");
    match LambdaMooDbParser::parse(Rule::object_def, object3_content) {
        Ok(mut pairs) => {
            if let Some(pair) = pairs.next() {
                println!("✅ Object #3 parsed");
                println!("Consumed: {} chars out of {}", pair.as_str().len(), object3_content.len());
                
                if pair.as_str().len() == object3_content.len() {
                    println!("✅ Consumed exactly all content");
                } else {
                    println!("🚨 Length mismatch!");
                    println!("Consumed content:");
                    for (i, line) in pair.as_str().lines().enumerate() {
                        println!("  Line {}: '{}'", i+1, line);
                    }
                }
            }
        }
        Err(e) => println!("❌ Object #3 failed: {}", e),
    }
    
    // Test what should be the correct object #3 content (without the extra 0)
    let correct_object3 = "#3\nWizard\n\n7\n3\n2\n-1\n-1\n1\n-1\n-1\n0\n0";
    
    println!("\n=== Testing correct object #3 (without extra 0) ===");
    match LambdaMooDbParser::parse(Rule::object_def, correct_object3) {
        Ok(mut pairs) => {
            if let Some(pair) = pairs.next() {
                println!("✅ Correct object #3 parsed");
                println!("Consumed: {} chars", pair.as_str().len());
            }
        }
        Err(e) => println!("❌ Correct object #3 failed: {}", e),
    }
    
    // Test what happens if we parse object #3 with extra content after
    let object3_with_extra = "#3\nWizard\n\n7\n3\n2\n-1\n-1\n1\n-1\n-1\n0\n0\n0\n#0:0\ncallers() && raise(E_PERM);";
    
    println!("\n=== Testing object #3 with extra content ===");
    match LambdaMooDbParser::parse(Rule::object_def, object3_with_extra) {
        Ok(mut pairs) => {
            if let Some(pair) = pairs.next() {
                println!("Consumed: {} chars out of {}", pair.as_str().len(), object3_with_extra.len());
                
                let consumed_lines = pair.as_str().lines().count();
                let total_lines = object3_with_extra.lines().count();
                println!("Consumed {} lines out of {}", consumed_lines, total_lines);
                
                if consumed_lines > 14 {  // Object #3 should be 14 lines
                    println!("🚨 PROBLEM: Consumed too many lines!");
                    println!("Last few consumed lines:");
                    let lines: Vec<&str> = pair.as_str().lines().collect();
                    for line in lines.iter().rev().take(3).rev() {
                        println!("  '{}'", line);
                    }
                }
            }
        }
        Err(e) => println!("❌ Object #3 with extra failed: {}", e),
    }
    
    Ok(())
}