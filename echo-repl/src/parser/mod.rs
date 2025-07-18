use anyhow::{Result, anyhow};

pub mod ast;

pub struct EchoParser {
    // Will be replaced with rust-sitter parser once grammar is compiled
}

impl EchoParser {
    pub fn new() -> Result<Self> {
        // Note: In production, we'd load the compiled grammar
        // For now, we'll implement a basic parser without rust-sitter
        // until we can properly compile the grammar
        Ok(Self {})
    }
    
    pub fn parse(&mut self, input: &str) -> Result<EchoAst> {
        // Temporary implementation - replace with rust-sitter parsing
        EchoAst::parse_simple(input)
    }
}

/// Simple AST representation for Echo
#[derive(Debug, Clone, PartialEq)]
pub enum EchoAst {
    // Literals
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    Null,
    
    // Variables and identifiers
    Identifier(String),
    
    // Expressions
    BinaryOp {
        op: BinaryOperator,
        left: Box<EchoAst>,
        right: Box<EchoAst>,
    },
    
    // Method call: obj:verb(args)
    MethodCall {
        object: Box<EchoAst>,
        verb: String,
        args: Vec<EchoAst>,
    },
    
    // Property access: obj.prop
    PropertyAccess {
        object: Box<EchoAst>,
        property: String,
    },
    
    // Statements
    Let {
        name: String,
        value: Box<EchoAst>,
    },
    
    // Object definition
    ObjectDef {
        name: String,
        parent: Option<String>,
        members: Vec<ObjectMember>,
    },
    
    // Top level
    Program(Vec<EchoAst>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOperator {
    Add, Sub, Mul, Div, Mod,
    Eq, Ne, Lt, Gt, Le, Ge,
    And, Or,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerbSignature {
    pub dobj: String,
    pub prep: String,
    pub iobj: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ObjectMember {
    Property {
        name: String,
        value: Option<EchoAst>,
    },
    Verb {
        name: String,
        signature: VerbSignature,
        code: String,
    },
    Function {
        name: String,
        params: Vec<String>,
        body: String,
    },
}

impl EchoAst {
    /// Simple parser for basic expressions - temporary until rust-sitter is integrated
    pub fn parse_simple(input: &str) -> Result<Self> {
        let trimmed = input.trim();
        
        // Very basic parsing for testing
        if trimmed.starts_with("let ") {
            let parts: Vec<&str> = trimmed[4..].split('=').collect();
            if parts.len() == 2 {
                let name = parts[0].trim().trim_end_matches(';').to_string();
                let value_str = parts[1].trim().trim_end_matches(';');
                
                if let Ok(num) = value_str.parse::<i64>() {
                    return Ok(EchoAst::Let {
                        name,
                        value: Box::new(EchoAst::Integer(num)),
                    });
                } else if value_str.starts_with('"') && value_str.ends_with('"') {
                    // String literal
                    return Ok(EchoAst::Let {
                        name,
                        value: Box::new(EchoAst::String(value_str[1..value_str.len()-1].to_string())),
                    });
                } else {
                    // Identifier
                    return Ok(EchoAst::Let {
                        name,
                        value: Box::new(EchoAst::Identifier(value_str.to_string())),
                    });
                }
            }
        } else if trimmed.starts_with("object ") {
            // Basic object parsing
            let lines: Vec<&str> = input.lines().map(|l| l.trim()).filter(|l| !l.is_empty()).collect();
            if lines.len() >= 2 && lines.last() == Some(&"endobject") {
                let header_parts: Vec<&str> = lines[0].split_whitespace().collect();
                if header_parts.len() >= 2 {
                    let name = header_parts[1].to_string();
                    let parent = if header_parts.len() >= 4 && header_parts[2] == "extends" {
                        Some(header_parts[3].to_string())
                    } else {
                        None
                    };
                    
                    let mut members = Vec::new();
                    let mut i = 1;
                    while i < lines.len() - 1 {
                        let line = lines[i].trim();
                        if line.starts_with("property ") {
                            let prop_parts: Vec<&str> = line[9..].split('=').collect();
                            if prop_parts.len() >= 2 {
                                let prop_name = prop_parts[0].trim().trim_end_matches(';').to_string();
                                let prop_value = prop_parts[1].trim().trim_end_matches(';');
                                
                                let value_ast = if prop_value.starts_with('"') && prop_value.ends_with('"') {
                                    EchoAst::String(prop_value[1..prop_value.len()-1].to_string())
                                } else if let Ok(num) = prop_value.parse::<i64>() {
                                    EchoAst::Integer(num)
                                } else {
                                    EchoAst::String(prop_value.to_string())
                                };
                                
                                members.push(ObjectMember::Property {
                                    name: prop_name,
                                    value: Some(value_ast),
                                });
                            }
                        } else if line.starts_with("verb ") {
                            // Parse verb definition
                            let verb_line = &line[5..];
                            if let Some(paren_start) = verb_line.find('(') {
                                let verb_name = verb_line[..paren_start].trim().trim_matches('"').to_string();
                                // Find endverb
                                let mut verb_body = String::new();
                                i += 1;
                                while i < lines.len() - 1 && lines[i].trim() != "endverb" {
                                    verb_body.push_str(lines[i]);
                                    verb_body.push('\n');
                                    i += 1;
                                }
                                members.push(ObjectMember::Verb {
                                    name: verb_name,
                                    signature: VerbSignature {
                                        dobj: "this".to_string(),
                                        prep: "none".to_string(),
                                        iobj: "none".to_string(),
                                    },
                                    code: verb_body.trim().to_string(),
                                });
                            }
                        }
                        i += 1;
                    }
                    
                    return Ok(EchoAst::ObjectDef { name, parent, members });
                }
            }
        } else if let Some((left, right)) = trimmed.split_once('+') {
            // Simple addition parsing
            let left = left.trim().parse::<i64>().map(EchoAst::Integer)?;
            let right = right.trim().parse::<i64>().map(EchoAst::Integer)?;
            return Ok(EchoAst::BinaryOp {
                op: BinaryOperator::Add,
                left: Box::new(left),
                right: Box::new(right),
            });
        } else if let Some(colon_pos) = trimmed.find(':') {
            // Method call: obj:verb(args)
            let obj_str = trimmed[..colon_pos].trim();
            let rest = trimmed[colon_pos + 1..].trim();
            
            if let Some(paren_start) = rest.find('(') {
                if rest.ends_with(')') {
                    let verb = rest[..paren_start].trim().to_string();
                    let args_str = &rest[paren_start + 1..rest.len() - 1];
                    
                    // Simple arg parsing - just split by comma for now
                    let args: Vec<EchoAst> = if args_str.trim().is_empty() {
                        vec![]
                    } else {
                        args_str.split(',')
                            .map(|arg| {
                                let arg = arg.trim();
                                if let Ok(num) = arg.parse::<i64>() {
                                    EchoAst::Integer(num)
                                } else {
                                    EchoAst::Identifier(arg.to_string())
                                }
                            })
                            .collect()
                    };
                    
                    return Ok(EchoAst::MethodCall {
                        object: Box::new(EchoAst::Identifier(obj_str.to_string())),
                        verb,
                        args,
                    });
                }
            }
        } else if let Ok(num) = trimmed.parse::<i64>() {
            return Ok(EchoAst::Integer(num));
        } else {
            return Ok(EchoAst::Identifier(trimmed.to_string()));
        }
        
        Err(anyhow!("Unable to parse: {}", input))
    }
}