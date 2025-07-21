use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=src");
    
    // Build rust-sitter parser for Echo grammar
    rust_sitter_tool::build_parsers(&PathBuf::from("src/parser/echo/grammar.rs"));
}