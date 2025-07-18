// Remove unused imports - error handling will be re-implemented for rust-sitter

pub mod ast; // Keep old AST for reference during migration
pub mod grammar;

// Use rust-sitter AST as the primary AST
pub use grammar::{EchoAst, EchoParser, parse_echo};