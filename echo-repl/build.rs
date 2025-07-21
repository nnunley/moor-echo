use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=src");
    
    // Build rust-sitter parser
    rust_sitter_tool::build_parsers(&PathBuf::from("src/parser/echo/grammar.rs"));
    
    // TODO: Fix rust_sitter field type issues in improved grammar
    // rust_sitter_tool::build_parsers(&PathBuf::from("src/parser/echo/grammar_improved.rs"));
}
