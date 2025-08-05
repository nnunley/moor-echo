use anyhow::{anyhow, Result};
use regex::Regex;

use crate::ast::{EchoAst, ObjectMember};

/// Parser specifically for MOO object definition files
pub struct MooObjectParser {
    preprocessor: Option<super::moo_preprocessor::MooPreprocessor>,
}

impl MooObjectParser {
    pub fn new() -> Self {
        Self {
            preprocessor: None,
        }
    }
    
    pub fn with_preprocessor(mut self, preprocessor: super::moo_preprocessor::MooPreprocessor) -> Self {
        self.preprocessor = Some(preprocessor);
        self
    }
    
    /// Parse a MOO object definition file
    pub fn parse_object_file(&mut self, source: &str) -> Result<EchoAst> {
        // Preprocess if we have a preprocessor
        let source = if let Some(preprocessor) = &mut self.preprocessor {
            preprocessor.process(source)
        } else {
            source.to_string()
        };
        
        let lines: Vec<&str> = source.lines().collect();
        if lines.is_empty() {
            return Err(anyhow!("Empty object file"));
        }
        
        // First line should be "object NAME" or "object #N"
        let object_re = Regex::new(r"^\s*object\s+(.+)$").unwrap();
        let first_line = lines[0];
        
        let name = if let Some(captures) = object_re.captures(first_line) {
            let obj_spec = captures.get(1).unwrap().as_str().trim();
            
            // Check if it's a direct object reference like #1
            if obj_spec.starts_with('#') {
                if let Ok(num) = obj_spec[1..].parse::<i64>() {
                    format!("object_{}", num)
                } else {
                    return Err(anyhow!("Invalid object reference: {}", obj_spec));
                }
            } else {
                // It's a named object
                obj_spec.to_string()
            }
        } else {
            return Err(anyhow!("Invalid object definition: expected 'object NAME'"));
        };
        
        // Parse the rest of the file
        let mut members = Vec::new();
        let mut parent = None;
        let mut i = 1;
        
        while i < lines.len() {
            let line = lines[i].trim();
            
            // Skip empty lines
            if line.is_empty() {
                i += 1;
                continue;
            }
            
            // Property definition with metadata
            if line.starts_with("property ") {
                if let Some(member) = self.parse_property_definition(line)? {
                    members.push(member);
                }
                i += 1;
                continue;
            }
            
            // Override property (simplified property without metadata)
            if line.starts_with("override ") {
                if let Some(member) = self.parse_override_property(line)? {
                    members.push(member);
                }
                i += 1;
                continue;
            }
            
            // Verb definition
            if line.starts_with("verb ") {
                let (verb_member, lines_consumed) = self.parse_verb_definition(&lines[i..])?;
                members.push(verb_member);
                i += lines_consumed;
                continue;
            }
            
            // Simple property assignment (name: value)
            if line.contains(':') && !line.contains("::") {
                let parts: Vec<&str> = line.splitn(2, ':').collect();
                if parts.len() == 2 {
                    let prop_name = parts[0].trim();
                    let prop_value = parts[1].trim();
                    
                    // Special handling for parent
                    if prop_name == "parent" {
                        parent = Some(prop_value.to_string());
                    } else {
                        // Parse the value as an expression
                        if let Ok(value) = self.parse_simple_value(prop_value) {
                            members.push(ObjectMember::Property {
                                name: prop_name.to_string(),
                                value,
                                permissions: None,
                                required_capabilities: Vec::new(),
                            });
                        }
                    }
                }
                i += 1;
                continue;
            }
            
            // If we can't parse this line, skip it
            i += 1;
        }
        
        // Always create an ObjectDef, even if we parsed an object reference
        // The name will be either the original name or "object_N" for #N
        Ok(EchoAst::ObjectDef {
            name,
            parent,
            members,
        })
    }
    
    fn parse_property_definition(&self, line: &str) -> Result<Option<ObjectMember>> {
        // Format: property name (owner: OWNER, flags: "FLAGS") = value;
        let prop_re = Regex::new(r"^\s*property\s+(\w+)\s*\([^)]+\)\s*=\s*(.+?);?\s*$").unwrap();
        
        if let Some(captures) = prop_re.captures(line) {
            let name = captures.get(1).unwrap().as_str().to_string();
            let value_str = captures.get(2).unwrap().as_str();
            
            let value = self.parse_simple_value(value_str)?;
            
            Ok(Some(ObjectMember::Property {
                name,
                value,
                permissions: None, // TODO: Parse permissions from metadata
                required_capabilities: Vec::new(),
            }))
        } else {
            Ok(None)
        }
    }
    
    fn parse_override_property(&self, line: &str) -> Result<Option<ObjectMember>> {
        // Format: override name = value;
        let prop_re = Regex::new(r"^\s*override\s+(\w+)\s*=\s*(.+?);?\s*$").unwrap();
        
        if let Some(captures) = prop_re.captures(line) {
            let name = captures.get(1).unwrap().as_str().to_string();
            let value_str = captures.get(2).unwrap().as_str();
            
            let value = self.parse_simple_value(value_str)?;
            
            Ok(Some(ObjectMember::Property {
                name,
                value,
                permissions: None,
                required_capabilities: Vec::new(),
            }))
        } else {
            Ok(None)
        }
    }
    
    fn parse_verb_definition(&self, lines: &[&str]) -> Result<(ObjectMember, usize)> {
        // First line: verb name (args) owner: OWNER flags: "FLAGS"
        let verb_re = Regex::new(r#"^\s*verb\s+"?([^"(]+)"?\s*\(([^)]+)\)"#).unwrap();
        
        if lines.is_empty() {
            return Err(anyhow!("Empty verb definition"));
        }
        
        let first_line = lines[0];
        let (name, _args_str) = if let Some(captures) = verb_re.captures(first_line) {
            let name = captures.get(1).unwrap().as_str().trim();
            let args = captures.get(2).unwrap().as_str();
            (name.to_string(), args.to_string())
        } else {
            return Err(anyhow!("Invalid verb definition: {}", first_line));
        };
        
        // Find the endverb
        let mut body_lines = Vec::new();
        let mut i = 1;
        while i < lines.len() {
            let line = lines[i];
            if line.trim() == "endverb" {
                break;
            }
            body_lines.push(line);
            i += 1;
        }
        
        if i >= lines.len() {
            return Err(anyhow!("Missing endverb for verb {}", name));
        }
        
        // For now, just store the body as a string
        let body_str = body_lines.join("\n");
        let body = vec![EchoAst::String(body_str)];
        
        let verb_member = ObjectMember::Verb {
            name,
            args: Vec::new(), // TODO: Parse parameters properly
            body,
            permissions: None,
            required_capabilities: Vec::new(),
        };
        
        Ok((verb_member, i + 1)) // +1 for the endverb line
    }
    
    fn parse_simple_value(&self, value: &str) -> Result<EchoAst> {
        let value = value.trim();
        
        // String literal
        if value.starts_with('"') && value.ends_with('"') {
            let content = &value[1..value.len()-1];
            return Ok(EchoAst::String(content.to_string()));
        }
        
        // Object reference
        if value.starts_with('#') {
            if let Ok(num) = value[1..].parse::<i64>() {
                return Ok(EchoAst::ObjectRef(num));
            }
        }
        
        // Boolean
        if value == "true" {
            return Ok(EchoAst::Boolean(true));
        }
        if value == "false" {
            return Ok(EchoAst::Boolean(false));
        }
        
        // Number
        if let Ok(num) = value.parse::<i64>() {
            return Ok(EchoAst::Number(num));
        }
        
        // Float
        if let Ok(f) = value.parse::<f64>() {
            return Ok(EchoAst::Float(f));
        }
        
        // Empty object literal {}
        if value == "{}" {
            return Ok(EchoAst::Map { entries: Vec::new() });
        }
        
        // Default to identifier
        Ok(EchoAst::Identifier(value.to_string()))
    }
}