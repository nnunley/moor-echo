// Simple multi-line detection for REPL
use crate::ast::EchoAst;
use crate::parser::Parser;

pub struct MultiLineCollector {
    lines: Vec<String>,
    is_collecting: bool,
    primary_construct: Option<String>,
    depth: usize,
}

impl MultiLineCollector {
    pub fn new() -> Self {
        Self {
            lines: Vec::new(),
            is_collecting: false,
            primary_construct: None,
            depth: 0,
        }
    }

    pub fn process_line(&mut self, line: &str, parser: &mut dyn Parser) -> LineProcessResult {
        let trimmed = line.trim();
        
        // Skip comments
        if trimmed.starts_with("//") || trimmed.starts_with("#") {
            return LineProcessResult::Complete(line.to_string());
        }
        
        // If we're not collecting, check if this starts a multi-line construct
        if !self.is_collecting {
            if let Some(construct) = self.get_construct_type(trimmed) {
                self.lines.clear();
                self.lines.push(line.to_string());
                self.is_collecting = true;
                self.primary_construct = Some(construct);
                self.depth = 1;
                return LineProcessResult::NeedMore;
            } else {
                // Single line - parse directly
                return LineProcessResult::Complete(line.to_string());
            }
        }
        
        // We're collecting lines
        self.lines.push(line.to_string());
        
        // Check for nested constructs
        if let Some(_) = self.get_construct_type(trimmed) {
            self.depth += 1;
        }
        
        // Check if this ends a construct
        if let Some(construct) = &self.primary_construct {
            if self.is_matching_end(construct, trimmed) {
                self.depth -= 1;
                if self.depth == 0 {
                    let complete_code = self.lines.join("\n");
                    self.lines.clear();
                    self.is_collecting = false;
                    self.primary_construct = None;
                    return LineProcessResult::Complete(complete_code);
                }
            }
        }
        
        // Don't try to parse control flow early - wait for end markers
        
        LineProcessResult::NeedMore
    }

    fn get_construct_type(&self, line: &str) -> Option<String> {
        if line.starts_with("object ") {
            Some("object".to_string())
        } else if line.starts_with("while ") {
            Some("while".to_string())
        } else if line.starts_with("for ") {
            Some("for".to_string())
        } else if line.starts_with("if ") {
            Some("if".to_string())
        } else if line.starts_with("fn ") {
            Some("fn".to_string())
        } else if line.contains(" = fn ") && line.contains('{') && !line.contains("endfn") {
            Some("fn".to_string())
        } else {
            None
        }
    }

    fn is_matching_end(&self, construct: &str, line: &str) -> bool {
        match construct {
            "object" => line == "endobject",
            "while" => line == "endwhile",
            "for" => line == "endfor",
            "if" => line == "endif",
            "fn" => line == "endfn",
            _ => false,
        }
    }


    pub fn is_collecting(&self) -> bool {
        self.is_collecting
    }

    pub fn reset(&mut self) {
        self.lines.clear();
        self.is_collecting = false;
        self.primary_construct = None;
        self.depth = 0;
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