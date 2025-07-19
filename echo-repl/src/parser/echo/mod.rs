// Modern Echo parser implementation
pub mod grammar;

use anyhow::Result;
use crate::ast::{self, LValue, BindingType, BindingPattern};

pub struct EchoParser {
    inner: grammar::EchoParser,
}

impl EchoParser {
    pub fn new() -> Result<Self> {
        Ok(Self {
            inner: grammar::EchoParser::new()?,
        })
    }
    
    pub fn parse(&mut self, input: &str) -> Result<ast::EchoAst> {
        // Check if this is an assignment statement
        if let Some(equals_pos) = input.find('=') {
            // Check if it's not == (equality comparison)
            if !input[equals_pos..].starts_with("==") {
                // Try to parse as assignment
                let target = input[..equals_pos].trim();
                let value = input[equals_pos + 1..].trim();
                
                // Parse the left side to determine the LValue type
                let lvalue = if let Some(dot_pos) = target.rfind('.') {
                    // Property assignment: obj.prop = value
                    let obj_expr = target[..dot_pos].trim();
                    let prop_name = target[dot_pos + 1..].trim();
                    
                    // Validate property name
                    if prop_name.chars().all(|c| c.is_alphanumeric() || c == '_') &&
                       prop_name.chars().next().map_or(false, |c| c.is_alphabetic() || c == '_') {
                        // Parse the object expression
                        let obj_ast = self.inner.parse(obj_expr)?;
                        let converted_obj = convert_grammar_to_ast(obj_ast)?;
                        
                        LValue::PropertyAccess {
                            object: Box::new(converted_obj),
                            property: prop_name.to_string(),
                        }
                    } else {
                        // Invalid property name, fall through to normal parsing
                        let grammar_ast = self.inner.parse(input)?;
                        return convert_grammar_to_ast(grammar_ast);
                    }
                } else {
                    // Check for let/const binding or simple assignment
                    let (binding_type, pattern_str) = if target.starts_with("let ") {
                        (BindingType::Let, target[4..].trim())
                    } else if target.starts_with("const ") {
                        (BindingType::Const, target[6..].trim())
                    } else {
                        (BindingType::None, target)
                    };
                    
                    // Parse the pattern
                    if let Some(pattern) = parse_binding_pattern(pattern_str) {
                        LValue::Binding {
                            binding_type,
                            pattern,
                        }
                    } else {
                        // Not a valid assignment target, fall through to normal parsing
                        let grammar_ast = self.inner.parse(input)?;
                        return convert_grammar_to_ast(grammar_ast);
                    }
                };
                
                // Parse the value expression
                let value_ast = self.inner.parse(value)?;
                let converted_value = convert_grammar_to_ast(value_ast)?;
                
                return Ok(ast::EchoAst::Assignment {
                    target: lvalue,
                    value: Box::new(converted_value),
                });
            }
        }
        
        // Otherwise, parse normally
        let grammar_ast = self.inner.parse(input)?;
        
        // Convert grammar AST to unified AST
        convert_grammar_to_ast(grammar_ast)
    }
}

impl super::Parser for EchoParser {
    fn parse(&mut self, source: &str) -> Result<ast::EchoAst> {
        self.parse(source)
    }
    
    fn parse_program(&mut self, source: &str) -> Result<ast::EchoAst> {
        // For now, let's try a simpler approach: split on statement boundaries
        let mut statements = Vec::new();
        let mut current_statement = String::new();
        let mut in_object = false;
        
        for line in source.lines() {
            let trimmed = line.trim();
            
            // Track object definitions which are multi-line
            if trimmed.starts_with("object ") {
                in_object = true;
            }
            
            if in_object {
                current_statement.push_str(line);
                current_statement.push('\n');
                
                if trimmed == "endobject" {
                    // Complete object definition
                    in_object = false;
                    match self.parse(&current_statement) {
                        Ok(stmt) => statements.push(stmt),
                        Err(e) => return Err(e),
                    }
                    current_statement.clear();
                }
            } else if !trimmed.is_empty() {
                // Single-line statement
                match self.parse(trimmed) {
                    Ok(stmt) => statements.push(stmt),
                    Err(e) => return Err(e),
                }
            }
        }
        
        // If we have an incomplete statement, try to parse it
        if !current_statement.is_empty() {
            match self.parse(&current_statement) {
                Ok(stmt) => statements.push(stmt),
                Err(e) => return Err(e),
            }
        }
        
        // Return the result
        if statements.is_empty() {
            Ok(ast::EchoAst::Null)
        } else if statements.len() == 1 {
            Ok(statements.into_iter().next().unwrap())
        } else {
            Ok(ast::EchoAst::Program(statements))
        }
    }
    
    fn name(&self) -> &'static str {
        "echo"
    }
}

/// Convert rust-sitter grammar AST to unified AST
fn convert_grammar_to_ast(node: grammar::EchoAst) -> Result<ast::EchoAst> {
    use grammar::EchoAst as G;
    use ast::EchoAst as A;
    
    Ok(match node {
        G::Number(n) => A::Number(n),
        G::Float(f) => A::Float(f),
        G::String(s) => A::String(s),
        G::True => A::Boolean(true),
        G::False => A::Boolean(false),
        G::Identifier(s) => A::Identifier(s),
        G::SysProp(s) => A::SystemProperty(s),
        G::ObjectRef(n) => A::ObjectRef(n),
        
        G::Add { left, right, .. } => A::Add {
            left: Box::new(convert_grammar_to_ast(*left)?),
            right: Box::new(convert_grammar_to_ast(*right)?),
        },
        
        G::Subtract { left, right, .. } => A::Subtract {
            left: Box::new(convert_grammar_to_ast(*left)?),
            right: Box::new(convert_grammar_to_ast(*right)?),
        },
        
        G::Multiply { left, right, .. } => A::Multiply {
            left: Box::new(convert_grammar_to_ast(*left)?),
            right: Box::new(convert_grammar_to_ast(*right)?),
        },
        
        G::Divide { left, right, .. } => A::Divide {
            left: Box::new(convert_grammar_to_ast(*left)?),
            right: Box::new(convert_grammar_to_ast(*right)?),
        },
        
        G::Modulo { left, right, .. } => A::Modulo {
            left: Box::new(convert_grammar_to_ast(*left)?),
            right: Box::new(convert_grammar_to_ast(*right)?),
        },
        
        // G::Assignment { target, value, .. } => A::Assignment {
        //     target: Box::new(convert_grammar_to_ast(*target)?),
        //     value: Box::new(convert_grammar_to_ast(*value)?),
        // },
        
        G::Equal { left, right, .. } => A::Equal {
            left: Box::new(convert_grammar_to_ast(*left)?),
            right: Box::new(convert_grammar_to_ast(*right)?),
        },
        
        G::PropertyAccess { object, property, .. } => {
            // Extract property name from identifier
            let prop_name = match property.as_ref() {
                G::Identifier(s) => s.clone(),
                _ => anyhow::bail!("Property must be identifier"),
            };
            A::PropertyAccess {
                object: Box::new(convert_grammar_to_ast(*object)?),
                property: prop_name,
            }
        },
        
        G::MethodCall { object, method, args, .. } => {
            // Extract method name from identifier
            let method_name = match method.as_ref() {
                G::Identifier(s) => s.clone(),
                _ => anyhow::bail!("Method must be identifier"),
            };
            let converted_args = args.into_iter()
                .filter(|arg| !matches!(arg, G::Comma))
                .map(convert_grammar_to_ast)
                .collect::<Result<Vec<_>>>()?;
            A::MethodCall {
                object: Box::new(convert_grammar_to_ast(*object)?),
                method: method_name,
                args: converted_args,
            }
        },
        
        G::List { elements, .. } => {
            let converted_elements = elements.into_iter()
                .filter(|elem| !matches!(elem, G::Comma))
                .map(convert_grammar_to_ast)
                .collect::<Result<Vec<_>>>()?;
            A::List { elements: converted_elements }
        },
        
        G::Paren { expr, .. } => {
            // Parentheses are just for grouping, return inner expression
            convert_grammar_to_ast(*expr)?
        },
        
        G::ObjectDef { name, members, .. } => {
            let obj_name = match name.as_ref() {
                G::Identifier(s) => s.clone(),
                _ => anyhow::bail!("Object name must be identifier"),
            };
            
            let mut converted_members = Vec::new();
            for member in members {
                match member {
                    grammar::ObjectMember::PropertyDef { name, value, .. } => {
                        let prop_name = match name.as_ref() {
                            G::Identifier(s) => s.clone(),
                            _ => anyhow::bail!("Property name must be identifier"),
                        };
                        let prop_value = convert_grammar_to_ast(*value)?;
                        converted_members.push(ast::ObjectMember::Property {
                            name: prop_name,
                            value: prop_value,
                            permissions: None,
                        });
                    }
                }
            }
            
            A::ObjectDef {
                name: obj_name,
                parent: None,
                members: converted_members,
            }
        },
        
        G::Comma => {
            // Skip commas - they're just separators
            anyhow::bail!("Unexpected comma in AST conversion")
        }
    })
}

/// Parse a binding pattern from a string
fn parse_binding_pattern(input: &str) -> Option<BindingPattern> {
    let trimmed = input.trim();
    
    // Ignore pattern: _
    if trimmed == "_" {
        return Some(BindingPattern::Ignore);
    }
    
    // List destructuring: [a, b, c]
    if trimmed.starts_with('[') && trimmed.ends_with(']') {
        let inner = &trimmed[1..trimmed.len()-1];
        let mut patterns = Vec::new();
        
        // Simple split by comma for now
        for part in inner.split(',') {
            if let Some(pattern) = parse_binding_pattern(part.trim()) {
                patterns.push(pattern);
            } else {
                return None;
            }
        }
        
        return Some(BindingPattern::List(patterns));
    }
    
    // Rest pattern: ...rest
    if trimmed.starts_with("...") {
        let rest_part = &trimmed[3..];
        if let Some(pattern) = parse_binding_pattern(rest_part) {
            return Some(BindingPattern::Rest(Box::new(pattern)));
        }
    }
    
    // Simple identifier
    if is_valid_identifier(trimmed) {
        return Some(BindingPattern::Identifier(trimmed.to_string()));
    }
    
    None
}

/// Check if a string is a valid identifier
fn is_valid_identifier(s: &str) -> bool {
    !s.is_empty() &&
    s.chars().all(|c| c.is_alphanumeric() || c == '_') &&
    s.chars().next().map_or(false, |c| c.is_alphabetic() || c == '_')
}

// Re-export for backward compatibility
pub use grammar::parse_echo;