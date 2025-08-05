use std::fs;
use echo_core::parser::lambdamoo_db_parser::LambdaMooDbParser;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Debugging LambdaCore full database parsing with detailed error tracking...");
    
    let content = fs::read_to_string("examples/LambdaCore-latest.db")?;
    let lines: Vec<&str> = content.lines().collect();
    
    println!("LambdaCore: {} chars, {} lines", content.len(), lines.len());
    
    // Try to parse progressively larger chunks to find where it fails
    let test_sizes = vec![1000, 2000, 5000, 10000, 20000, 50000];
    
    for &size in &test_sizes {
        if size > lines.len() {
            continue;
        }
        
        println!("\n=== Testing first {} lines ===", size);
        
        let partial_content = lines[0..size].join("\n") + "\n";
        
        match LambdaMooDbParser::parse_database(&partial_content) {
            Ok(db) => {
                println!("✅ First {} lines parse successfully", size);
                println!("   Objects found: {}", db.objects.len());
                println!("   Verb programs: {}", db.verb_programs.len());
                
                // Show some object IDs
                let mut obj_ids: Vec<i64> = db.objects.keys().cloned().collect();
                obj_ids.sort();
                if obj_ids.len() > 0 {
                    println!("   Object IDs: {:?}", &obj_ids[..obj_ids.len().min(10)]);
                }
            }
            Err(e) => {
                println!("❌ First {} lines parsing failed: {}", size, e);
                println!("   Error details: {:?}", e);
                
                // Check error type
                let error_string = format!("{}", e);
                if error_string.contains("invalid digit") {
                    println!("   🎯 FOUND 'invalid digit' ERROR!");
                    println!("   This is a Rust numeric parsing error, not a grammar error");
                    
                    // Try to identify which numeric field is problematic
                    if error_string.contains("flags") {
                        println!("   Likely problematic field: flags");
                    } else if error_string.contains("owner") {
                        println!("   Likely problematic field: owner");
                    } else if error_string.contains("location") {
                        println!("   Likely problematic field: location");
                    } else if error_string.contains("contents") {
                        println!("   Likely problematic field: contents");
                    } else if error_string.contains("next") {
                        println!("   Likely problematic field: next");
                    } else if error_string.contains("parent") {
                        println!("   Likely problematic field: parent");
                    } else if error_string.contains("child") {
                        println!("   Likely problematic field: child");
                    } else if error_string.contains("sibling") {
                        println!("   Likely problematic field: sibling");
                    } else {
                        println!("   Field causing error not identified in message");
                    }
                }
                
                // Show the problematic area (lines around the failure point)
                if size > 1000 {
                    let previous_working_size = test_sizes.iter()
                        .rev()
                        .find(|&&s| s < size)
                        .unwrap_or(&1000);
                    
                    println!("   Problem is between lines {} and {}", previous_working_size, size);
                    
                    // Show some lines in that range
                    let check_start = *previous_working_size;
                    let check_end = (check_start + 50).min(size);
                    
                    println!("   Lines {}-{} (sample):", check_start + 1, check_end + 1);
                    for i in check_start..check_end {
                        if i < lines.len() {
                            println!("   {:5}: '{}'", i + 1, lines[i]);
                        }
                    }
                }
                
                break; // Stop at first failure
            }
        }
    }
    
    Ok(())
}