use std::fs;
use echo_core::parser::lambdamoo_db_parser::LambdaMooDbParser;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing full LambdaCore parsing with error details...");
    
    let file_content = fs::read_to_string("examples/LambdaCore-latest.db")?;
    println!("File size: {} chars, {} lines", file_content.len(), file_content.lines().count());
    
    // Try the actual parsing that the browser does
    match LambdaMooDbParser::parse_database(&file_content) {
        Ok(db) => {
            println!("✅ LambdaCore parsed successfully!");
            println!("Objects: {}", db.objects.len());
            println!("Verbs: {}", db.verb_programs.len());
        }
        Err(e) => {
            println!("❌ LambdaCore parsing failed: {}", e);
            
            // Print the error details
            println!("Error details: {:?}", e);
            
            // The error might contain information about which line/field failed
            let error_str = format!("{}", e);
            if error_str.contains("invalid digit") {
                println!("\nThis is a numeric parsing error, not a grammar error!");
                println!("Likely caused by trying to parse a non-numeric string as a number");
                println!("Common causes:");
                println!("  - Empty strings where numbers expected");
                println!("  - Strings with whitespace or special characters");
                println!("  - Object IDs with special formats");
            }
        }
    }
    
    Ok(())
}