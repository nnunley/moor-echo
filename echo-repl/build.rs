use std::path::Path;

fn main() {
    let grammar_path = Path::new("src/parser/grammar.js");
    
    // Only rebuild if grammar changed
    println!("cargo:rerun-if-changed={}", grammar_path.display());
    
    // For now, we'll use the grammar at runtime
    // In production, we'd compile it here
}