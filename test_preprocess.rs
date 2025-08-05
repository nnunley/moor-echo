use echo_core::parser::moo_preprocessor::MooPreprocessor;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut preprocessor = MooPreprocessor::new();
    
    // Load constants
    let constants = fs::read_to_string("/Users/ndn/development/cowbell/src/constants.moo")?;
    preprocessor.load_defines(&constants);
    
    // Process root.moo
    let root_source = fs::read_to_string("/Users/ndn/development/cowbell/src/root.moo")?;
    let processed = preprocessor.process(&root_source);
    
    println!("=== ORIGINAL ===");
    println!("{}", root_source);
    println!("\n=== PROCESSED ===");
    println!("{}", processed);
    
    Ok(())
}