use std::collections::HashMap;
use regex::Regex;

/// Preprocessor for MOO code that handles define substitutions
#[derive(Clone, Debug)]
pub struct MooPreprocessor {
    /// Maps constant names to their values
    defines: HashMap<String, String>,
}

impl MooPreprocessor {
    pub fn new() -> Self {
        Self {
            defines: HashMap::new(),
        }
    }
    
    /// Process a MOO source file, handling defines and substitutions
    pub fn process(&mut self, source: &str) -> String {
        let mut output = String::new();
        let mut lines = source.lines();
        
        // Regular expressions for parsing
        let define_re = Regex::new(r"^\s*define\s+([A-Z_][A-Z0-9_]*)\s*=\s*(.+?)\s*;?\s*$").unwrap();
        
        while let Some(line) = lines.next() {
            // Check if this is a define statement
            if let Some(captures) = define_re.captures(line) {
                let name = captures.get(1).unwrap().as_str();
                let value = captures.get(2).unwrap().as_str();
                self.defines.insert(name.to_string(), value.to_string());
                // Don't output define lines - they're preprocessor directives
                continue;
            }
            
            // For object definitions, keep the object name as-is
            // The MOO-to-Echo object mapping should be handled elsewhere
            if line.trim_start().starts_with("object ") {
                // Don't substitute object names - just pass through
                output.push_str(line);
                output.push('\n');
                continue;
            }
            
            // For all other lines, perform substitutions on uppercase identifiers
            let mut processed_line = line.to_string();
            
            // Replace all occurrences of defined constants
            for (name, value) in &self.defines {
                // Only replace whole words (not parts of identifiers)
                let pattern = format!(r"\b{}\b", regex::escape(name));
                if let Ok(re) = Regex::new(&pattern) {
                    processed_line = re.replace_all(&processed_line, value).to_string();
                }
            }
            
            output.push_str(&processed_line);
            output.push('\n');
        }
        
        output
    }
    
    /// Load defines from a constants file
    pub fn load_defines(&mut self, source: &str) {
        let define_re = Regex::new(r"^\s*define\s+([A-Z_][A-Z0-9_]*)\s*=\s*(.+?)\s*;?\s*$").unwrap();
        
        for line in source.lines() {
            if let Some(captures) = define_re.captures(line) {
                let name = captures.get(1).unwrap().as_str();
                let value = captures.get(2).unwrap().as_str();
                self.defines.insert(name.to_string(), value.to_string());
            }
        }
    }
    
    /// Get the current defines map
    pub fn defines(&self) -> &HashMap<String, String> {
        &self.defines
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_define_processing() {
        let mut preprocessor = MooPreprocessor::new();
        
        let source = r#"
define ROOT = #1;
define PLAYER = #4;

object ROOT
  name: "Root Object"
endobject

object PLAYER
  parent: ROOT
endobject
"#;
        
        let processed = preprocessor.process(source);
        
        // Defines should be removed and substitutions made
        assert!(!processed.contains("define"));
        assert!(processed.contains("object #1"));
        assert!(processed.contains("object #4"));
        assert!(processed.contains("parent: #1"));
    }
    
    #[test]
    fn test_load_defines() {
        let mut preprocessor = MooPreprocessor::new();
        
        let constants = r#"
define FAILED_MATCH = #-3;
define AMBIGUOUS = #-2;
define NOTHING = #-1;
define SYSOBJ = #0;
define ROOT = #1;
"#;
        
        preprocessor.load_defines(constants);
        
        assert_eq!(preprocessor.defines.len(), 5);
        assert_eq!(preprocessor.defines.get("ROOT"), Some(&"#1".to_string()));
        assert_eq!(preprocessor.defines.get("SYSOBJ"), Some(&"#0".to_string()));
        assert_eq!(preprocessor.defines.get("NOTHING"), Some(&"#-1".to_string()));
    }
}