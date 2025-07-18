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
pub enum ObjectMember {
    Property {
        name: String,
        value: Option<EchoAst>,
    },
    Verb {
        name: String,
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
                    for line in &lines[1..lines.len()-1] {
                        let line = line.trim();
                        if line.starts_with("property ") {
                            let prop_parts: Vec<&str> = line[9..].split('=').collect();
                            if !prop_parts.is_empty() {
                                let prop_name = prop_parts[0].trim().trim_end_matches(';').to_string();
                                members.push(ObjectMember::Property {
                                    name: prop_name,
                                    value: Some(EchoAst::String("test".to_string())),
                                });
                            }
                        }
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
        } else if let Ok(num) = trimmed.parse::<i64>() {
            return Ok(EchoAst::Integer(num));
        } else {
            return Ok(EchoAst::Identifier(trimmed.to_string()));
        }
        
        Err(anyhow!("Unable to parse: {}", input))
    }
}