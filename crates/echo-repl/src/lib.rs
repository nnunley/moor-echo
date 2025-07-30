//! Echo REPL - Interactive command-line interface for the Echo programming
//! language
//!
//! This crate provides REPL (Read-Eval-Print Loop) functionality for Echo,
//! including command parsing, multi-line input handling, and player management.

pub mod repl;

// Re-export commonly used types for convenience
pub use repl::{DefaultNotifier, Repl, ReplCommand, ReplNotifier};
