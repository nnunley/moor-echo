// Simple multi-line detection for REPL
use crate::parser::Parser;

pub struct MultiLineCollector {
    lines: Vec<String>,
    is_collecting: bool,
}

impl MultiLineCollector {
    pub fn new() -> Self {
        Self {
            lines: Vec::new(),
            is_collecting: false,
        }
    }

    pub fn process_line(&mut self, line: &str, parser: &mut dyn Parser) -> LineProcessResult {
        let trimmed = line.trim();
        
        // Skip comments
        if trimmed.starts_with("//") || trimmed.starts_with("#") {
            return LineProcessResult::Complete(line.to_string());
        }
        
        // If we're not collecting, start a new potential multi-line input
        if !self.is_collecting {
            self.lines.clear();
            self.lines.push(line.to_string());
            
            // Try to parse the single line first
            match parser.parse(line) {
                Ok(_) => {
                    // It parsed successfully, so it's complete
                    return LineProcessResult::Complete(line.to_string());
                }
                Err(e) => {
                    // Check if the error indicates incomplete input
                    if self.is_incomplete_error(&e) {
                        self.is_collecting = true;
                        return LineProcessResult::NeedMore;
                    } else {
                        // It's a real parse error, not incomplete input
                        return LineProcessResult::Complete(line.to_string());
                    }
                }
            }
        }
        
        // We're collecting lines
        self.lines.push(line.to_string());
        
        // Try to parse the accumulated input
        let accumulated = self.lines.join("\n");
        match parser.parse(&accumulated) {
            Ok(_) => {
                // It parsed successfully, so we have complete input
                self.lines.clear();
                self.is_collecting = false;
                LineProcessResult::Complete(accumulated)
            }
            Err(e) => {
                // Check if it's still incomplete or a real error
                if self.is_incomplete_error(&e) {
                    LineProcessResult::NeedMore
                } else {
                    // Real parse error - return what we have
                    self.lines.clear();
                    self.is_collecting = false;
                    LineProcessResult::Complete(accumulated)
                }
            }
        }
    }

    fn is_incomplete_error(&self, error: &anyhow::Error) -> bool {
        let error_str = error.to_string().to_lowercase();
        
        // Check for common incomplete input indicators
        error_str.contains("unexpected end") ||
        error_str.contains("expected") && error_str.contains("but found end") ||
        error_str.contains("incomplete") ||
        error_str.contains("unfinished") ||
        error_str.contains("unclosed") ||
        error_str.contains("missing") && (error_str.contains("end") || error_str.contains("closing"))
    }


    pub fn is_collecting(&self) -> bool {
        self.is_collecting
    }

    pub fn reset(&mut self) {
        self.lines.clear();
        self.is_collecting = false;
    }

    pub fn get_prompt(&self) -> &'static str {
        if self.is_collecting {
            ".. "
        } else {
            ">> "
        }
    }
}

pub enum LineProcessResult {
    Complete(String),
    NeedMore,
}