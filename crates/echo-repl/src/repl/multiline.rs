//! Multi-line input collection for the REPL
//!
//! Handles intelligent collection of multi-line Echo code, including:
//! - Bracket/brace matching
//! - String literal handling
//! - Statement completion detection
//! - Prompt management

use echo_core::Parser;

/// Result of processing a line of input
#[derive(Debug)]
pub enum LineProcessResult {
    /// Input is complete and ready for execution
    Complete(String),
    /// More input is needed to complete the statement
    NeedMore,
}

/// Collects multi-line input for complete statements
pub struct MultiLineCollector {
    /// Buffer for collecting lines
    buffer: String,
    /// Current nesting level (for braces, brackets, etc.)
    nesting_level: i32,
    /// Whether we're inside a string literal
    in_string: bool,
    /// String delimiter character (single or double quote)
    string_delimiter: char,
    /// Whether the last character was an escape
    last_was_escape: bool,
}

impl MultiLineCollector {
    /// Create a new multi-line collector
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
            nesting_level: 0,
            in_string: false,
            string_delimiter: '"',
            last_was_escape: false,
        }
    }

    /// Get the appropriate prompt for the current state
    pub fn get_prompt(&self) -> &'static str {
        if self.is_collecting() {
            "   " // Continuation prompt
        } else {
            ">> " // Main prompt
        }
    }

    /// Check if we're currently collecting a multi-line statement
    pub fn is_collecting(&self) -> bool {
        !self.buffer.is_empty()
    }

    /// Reset the collector state
    pub fn reset(&mut self) {
        self.buffer.clear();
        self.nesting_level = 0;
        self.in_string = false;
        self.last_was_escape = false;
    }

    /// Process a line of input
    pub fn process_line(&mut self, line: &str, parser: &mut Box<dyn Parser>) -> LineProcessResult {
        // Add line to buffer
        if !self.buffer.is_empty() {
            self.buffer.push('\n');
        }
        self.buffer.push_str(line);

        // Update parsing state
        self.update_parsing_state(line);

        // Check if we have a complete statement
        if self.is_complete_statement(parser) {
            let complete_code = self.buffer.clone();
            self.reset();
            LineProcessResult::Complete(complete_code)
        } else {
            LineProcessResult::NeedMore
        }
    }

    /// Update parsing state based on the new line
    fn update_parsing_state(&mut self, line: &str) {
        for ch in line.chars() {
            if self.in_string {
                if self.last_was_escape {
                    self.last_was_escape = false;
                } else if ch == '\\' {
                    self.last_was_escape = true;
                } else if ch == self.string_delimiter {
                    self.in_string = false;
                }
            } else {
                match ch {
                    '"' | '\'' => {
                        self.in_string = true;
                        self.string_delimiter = ch;
                        self.last_was_escape = false;
                    }
                    '{' | '(' | '[' => {
                        self.nesting_level += 1;
                    }
                    '}' | ')' | ']' => {
                        self.nesting_level -= 1;
                    }
                    _ => {}
                }
            }
        }
    }

    /// Check if the current buffer contains a complete statement
    fn is_complete_statement(&self, parser: &mut Box<dyn Parser>) -> bool {
        // If we're inside a string or have unmatched brackets, not complete
        if self.in_string || self.nesting_level > 0 {
            return false;
        }

        // Try to parse the current buffer
        // If it parses successfully, it's complete
        match parser.parse(&self.buffer) {
            Ok(_) => true,
            Err(_) => {
                // If parsing fails, check if it's due to incomplete input
                // For now, we'll use a simple heuristic: if the line doesn't end with
                // a semicolon and doesn't look like a complete expression, continue
                let trimmed = self.buffer.trim();

                // Common patterns that suggest more input is needed
                if trimmed.ends_with(',')
                    || trimmed.ends_with('=')
                    || trimmed.ends_with("=>")
                    || trimmed.ends_with("if")
                    || trimmed.ends_with("else")
                    || trimmed.ends_with("for")
                    || trimmed.ends_with("while")
                    || trimmed.ends_with("function")
                    || trimmed.ends_with("let")
                    || trimmed.ends_with("const")
                {
                    return false;
                }

                // If it ends with a semicolon, probably complete
                if trimmed.ends_with(';') {
                    return true;
                }

                // For other cases, assume it's complete if nesting is balanced
                self.nesting_level == 0 && !self.in_string
            }
        }
    }
}

impl Default for MultiLineCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use echo_core::create_parser;

    use super::*;

    #[test]
    fn test_simple_complete_statement() {
        let mut collector = MultiLineCollector::new();
        let mut parser = create_parser("echo").unwrap();

        match collector.process_line("let x = 42;", &mut parser) {
            LineProcessResult::Complete(code) => assert_eq!(code, "let x = 42;"),
            LineProcessResult::NeedMore => panic!("Expected complete statement"),
        }
    }

    #[test]
    fn test_multiline_object() {
        let mut collector = MultiLineCollector::new();
        let mut parser = create_parser("echo").unwrap();

        // First line - should need more
        match collector.process_line("let obj = {", &mut parser) {
            LineProcessResult::NeedMore => {}
            LineProcessResult::Complete(_) => panic!("Expected need more"),
        }

        // Second line - should need more
        match collector.process_line("  name: \"test\",", &mut parser) {
            LineProcessResult::NeedMore => {}
            LineProcessResult::Complete(_) => panic!("Expected need more"),
        }

        // Third line - should be complete
        match collector.process_line("};", &mut parser) {
            LineProcessResult::Complete(code) => {
                assert!(code.contains("let obj = {"));
                assert!(code.contains("name: \"test\","));
                assert!(code.contains("};"));
            }
            LineProcessResult::NeedMore => panic!("Expected complete statement"),
        }
    }

    #[test]
    fn test_string_with_quotes() {
        let mut collector = MultiLineCollector::new();
        let mut parser = create_parser("echo").unwrap();

        match collector.process_line("let s = \"hello world\";", &mut parser) {
            LineProcessResult::Complete(code) => assert_eq!(code, "let s = \"hello world\";"),
            LineProcessResult::NeedMore => panic!("Expected complete statement"),
        }
    }
}
