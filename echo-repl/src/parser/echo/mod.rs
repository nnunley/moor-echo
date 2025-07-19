// Modern Echo parser implementation
pub mod grammar;

use anyhow::{Result, anyhow};
use crate::ast::{self, LValue, BindingType, BindingPattern};

// Helper function to extract parameter names from scatter expressions
fn extract_params_from_scatter(scatter: Box<grammar::EchoAst>) -> Result<Vec<String>> {
    use grammar::EchoAst as G;
    
    match *scatter {
        // Simple identifier: x
        G::Identifier(name) => Ok(vec![name]),
        
        // List pattern: {x, y, z}
        G::List { elements, .. } => {
            let mut params = Vec::new();
            for elem in elements {
                match elem {
                    G::Identifier(name) => params.push(name),
                    _ => return Err(anyhow!("Only identifiers allowed in parameter list")),
                }
            }
            Ok(params)
        },
        
        // Parenthesized expression: (x) or (x, y)
        G::Paren { expr, .. } => extract_params_from_scatter(expr),
        
        _ => Err(anyhow!("Invalid parameter pattern")),
    }
}

// Helper function to extract parameter names from ParamPattern
fn extract_params_from_pattern(pattern: grammar::ParamPattern) -> Vec<String> {
    use grammar::{ParamPattern as P, ParamElement as E};
    
    match pattern {
        P::Single(elem) => match elem {
            E::Simple(ident) => vec![ident.name],
            E::Optional { name, .. } => vec![name.name],
            E::Rest { name, .. } => vec![name.name],
        },
        P::Multiple { params, .. } => {
            params.into_iter().map(|elem| match elem {
                E::Simple(ident) => ident.name,
                E::Optional { name, .. } => name.name,
                E::Rest { name, .. } => name.name,
            }).collect()
        },
    }
}

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
        // Split on statement boundaries, handling multi-line constructs
        let mut statements = Vec::new();
        let mut current_statement = String::new();
        let mut in_multiline = false;
        let mut multiline_type = "";
        
        for line in source.lines() {
            let trimmed = line.trim();
            
            // Skip empty lines and comments
            if trimmed.is_empty() || trimmed.starts_with("#") {
                continue;
            }
            
            // Check for start of multi-line constructs
            if !in_multiline {
                if trimmed.starts_with("object ") {
                    in_multiline = true;
                    multiline_type = "object";
                } else if trimmed.starts_with("while ") {
                    in_multiline = true;
                    multiline_type = "while";
                } else if trimmed.starts_with("for ") {
                    in_multiline = true;
                    multiline_type = "for";
                } else if trimmed.starts_with("fn ") {
                    in_multiline = true;
                    multiline_type = "fn";
                } else if trimmed.starts_with("if ") {
                    in_multiline = true;
                    multiline_type = "if";
                }
            }
            
            if in_multiline {
                current_statement.push_str(line);
                current_statement.push('\n');
                
                // Check for end of multi-line construct
                let should_end = match multiline_type {
                    "object" => trimmed == "endobject",
                    "fn" => trimmed == "endfn",
                    "while" | "for" | "if" => {
                        // For control flow, we collect until we have a complete statement
                        // Try parsing what we have so far
                        self.parse(&current_statement).is_ok()
                    }
                    _ => false,
                };
                
                if should_end {
                    in_multiline = false;
                    match self.parse(&current_statement) {
                        Ok(stmt) => statements.push(stmt),
                        Err(e) => return Err(e),
                    }
                    current_statement.clear();
                    multiline_type = "";
                }
            } else {
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
        
        G::Assignment { target, value, .. } => {
            // Convert to unified AST assignment format
            // We need to handle the target as an LValue
            match convert_grammar_to_ast(*target)? {
                ast::EchoAst::Identifier(name) => {
                    ast::EchoAst::Assignment {
                        target: ast::LValue::Binding {
                            binding_type: ast::BindingType::None,
                            pattern: ast::BindingPattern::Identifier(name),
                        },
                        value: Box::new(convert_grammar_to_ast(*value)?),
                    }
                }
                _ => return Err(anyhow!("Assignment target must be an identifier")),
            }
        },
        
        G::Equal { left, right, .. } => A::Equal {
            left: Box::new(convert_grammar_to_ast(*left)?),
            right: Box::new(convert_grammar_to_ast(*right)?),
        },
        
        G::NotEqual { left, right, .. } => A::NotEqual {
            left: Box::new(convert_grammar_to_ast(*left)?),
            right: Box::new(convert_grammar_to_ast(*right)?),
        },
        
        G::LessThan { left, right, .. } => A::LessThan {
            left: Box::new(convert_grammar_to_ast(*left)?),
            right: Box::new(convert_grammar_to_ast(*right)?),
        },
        
        G::LessEqual { left, right, .. } => A::LessEqual {
            left: Box::new(convert_grammar_to_ast(*left)?),
            right: Box::new(convert_grammar_to_ast(*right)?),
        },
        
        G::GreaterThan { left, right, .. } => A::GreaterThan {
            left: Box::new(convert_grammar_to_ast(*left)?),
            right: Box::new(convert_grammar_to_ast(*right)?),
        },
        
        G::GreaterEqual { left, right, .. } => A::GreaterEqual {
            left: Box::new(convert_grammar_to_ast(*left)?),
            right: Box::new(convert_grammar_to_ast(*right)?),
        },
        
        G::And { left, right, .. } => A::And {
            left: Box::new(convert_grammar_to_ast(*left)?),
            right: Box::new(convert_grammar_to_ast(*right)?),
        },
        
        G::Or { left, right, .. } => A::Or {
            left: Box::new(convert_grammar_to_ast(*left)?),
            right: Box::new(convert_grammar_to_ast(*right)?),
        },
        
        G::Not { operand, .. } => A::Not {
            operand: Box::new(convert_grammar_to_ast(*operand)?),
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
            let method_name = method.name;
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
        
        G::ArrowFunction { params, body, .. } => {
            // Extract parameter names from the scatter expression
            let param_names = extract_params_from_scatter(params)?;
            A::Lambda {
                params: param_names,
                body: Box::new(convert_grammar_to_ast(*body)?),
            }
        },
        
        G::BlockFunction { params, body, .. } => {
            // Extract parameter names from the parameter pattern
            let param_names = extract_params_from_pattern(params);
            // Convert body to a single expression (Program)
            let body_stmts = body.into_iter()
                .map(convert_grammar_to_ast)
                .collect::<Result<Vec<_>>>()?;
            A::Lambda {
                params: param_names,
                body: Box::new(A::Program(body_stmts)),
            }
        },
        
        G::Call { func, args, .. } => {
            let converted_func = Box::new(convert_grammar_to_ast(*func)?);
            let converted_args = args.into_iter()
                .map(convert_grammar_to_ast)
                .collect::<Result<Vec<_>>>()?;
            A::Call {
                func: converted_func,
                args: converted_args,
            }
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
                        let prop_name = name.name.clone();
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
        
        G::If { condition, then_body, else_clause, .. } => {
            // If with optional else clause using MOO syntax
            let then_vec = then_body.into_iter()
                .map(convert_grammar_to_ast)
                .collect::<Result<Vec<_>>>()?;
            let else_vec = if let Some(else_clause) = else_clause {
                Some(else_clause.body.into_iter()
                    .map(convert_grammar_to_ast)
                    .collect::<Result<Vec<_>>>()?)
            } else {
                None
            };
            
            A::If {
                condition: Box::new(convert_grammar_to_ast(*condition)?),
                then_branch: then_vec,
                else_branch: else_vec,
            }
        }
        
        G::While { condition, body, .. } => {
            let body_vec = body.into_iter()
                .map(convert_grammar_to_ast)
                .collect::<Result<Vec<_>>>()?;
            
            A::While {
                label: None, // Labels will be added later as separate feature
                condition: Box::new(convert_grammar_to_ast(*condition)?),
                body: body_vec,
            }
        }
        
        G::For { variable, collection, body, .. } => {
            let var_name = match variable.as_ref() {
                G::Identifier(s) => s.clone(),
                _ => anyhow::bail!("For loop variable must be identifier"),
            };
            
            let body_vec = body.into_iter()
                .map(convert_grammar_to_ast)
                .collect::<Result<Vec<_>>>()?;
            
            A::For {
                label: None, // For loops don't have labels in our grammar yet
                variable: var_name,
                collection: Box::new(convert_grammar_to_ast(*collection)?),
                body: body_vec,
            }
        }
        
        G::Break { label, .. } => {
            let label_str = if let Some(label_expr) = label {
                match label_expr.as_ref() {
                    G::Identifier(s) => Some(s.clone()),
                    _ => anyhow::bail!("Break label must be identifier"),
                }
            } else {
                None
            };
            
            A::Break { label: label_str }
        }
        
        G::Continue { label, .. } => {
            let label_str = if let Some(label_expr) = label {
                match label_expr.as_ref() {
                    G::Identifier(s) => Some(s.clone()),
                    _ => anyhow::bail!("Continue label must be identifier"),
                }
            } else {
                None
            };
            
            A::Continue { label: label_str }
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