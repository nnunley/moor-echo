// Parser module - supports multiple parser implementations
use std::path::Path;

use anyhow::Result;

use crate::ast::EchoAst;

pub mod echo;
// pub mod moo_compat;  // TODO: Implement MOO compatibility parser
pub mod simple_parser;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod simple_parser_tests;

#[cfg(test)]
mod test_basic;

/// Trait for all Echo parsers
pub trait Parser: Send + Sync {
    /// Parse source code into Echo AST
    fn parse(&mut self, source: &str) -> Result<EchoAst>;

    /// Parse source code as a program (multiple statements)
    fn parse_program(&mut self, source: &str) -> Result<EchoAst> {
        // Default implementation wraps result in Program node
        match self.parse(source) {
            Ok(EchoAst::Program(_)) => self.parse(source),
            Ok(expr) => Ok(EchoAst::Program(vec![expr])),
            Err(e) => Err(e),
        }
    }

    /// Parse a file
    fn parse_file(&mut self, path: &Path) -> Result<EchoAst> {
        let source = std::fs::read_to_string(path)?;
        self.parse(&source)
    }

    /// Get parser name for debugging
    fn name(&self) -> &'static str;
}

/// Parser types available
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParserType {
    /// Modern Echo syntax
    Echo,
    /// MOO compatibility mode
    Moo,
    /// Auto-detect based on file extension or content
    Auto,
}

// Re-export for backward compatibility
pub use echo::EchoParser;
// TEMPORARILY DISABLED due to rust-sitter dependency issues
// pub use echo::grammar::{EchoAst as GrammarAst, ObjectMember as
// GrammarObjectMember}; pub use echo::parse_echo;

/// Create a parser based on type
pub fn create_parser(parser_type: &str) -> Result<Box<dyn Parser>> {
    match parser_type {
        "echo" => {
            // Use the rust-sitter parser
            Ok(Box::new(echo::EchoParser::new()?))
        }
        // "moo" => Ok(Box::new(MooCompatParser::new()?)),
        _ => anyhow::bail!("Unknown parser type: {}", parser_type),
    }
}
