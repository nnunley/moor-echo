// Simple parser implementation using the unified AST
// This replaces the duplicate AST implementation that was in parser/ast.rs

use crate::ast::{BindingPattern, BindingType, EchoAst, LValue, ObjectMember};

/// Simple parser for basic Echo expressions
/// This is a temporary implementation until rust-sitter is fully integrated
pub fn parse_simple(input: &str) -> Result<EchoAst, String> {
    let trimmed = input.trim();

    if trimmed.is_empty() {
        return Err("Empty input".to_string());
    }

    // Handle let statements
    if trimmed.starts_with("let ") {
        return parse_let_statement(trimmed);
    }

    // Handle const statements
    if trimmed.starts_with("const ") {
        return parse_const_statement(trimmed);
    }

    // Handle object definitions
    if trimmed.starts_with("object ") {
        return parse_object_definition(input);
    }

    // Handle assignments (property or simple)
    if trimmed.contains('=') && !trimmed.contains("==") {
        return parse_assignment(trimmed);
    }

    // Handle method calls
    if let Some(colon_pos) = trimmed.find(':') {
        if trimmed[colon_pos + 1..].contains('(') {
            return parse_method_call(trimmed);
        }
    }

    // Handle property access
    if trimmed.contains('.') && !trimmed.contains("..") {
        let dot_pos = trimmed.find('.').unwrap();
        let after_dot = &trimmed[dot_pos + 1..];

        // Check if it's a binary operation with property access
        if after_dot.chars().any(|c| "+-*/".contains(c)) {
            return parse_binary_with_property(trimmed);
        }

        // Simple property access
        if !after_dot.contains('(') && !after_dot.is_empty() {
            return parse_property_access(trimmed);
        }
    }

    // Handle lists
    if trimmed.starts_with('[') && trimmed.ends_with(']') {
        return parse_list(trimmed);
    }

    // Handle boolean operations  
    if trimmed.contains("&&") || trimmed.contains("||") {
        return parse_boolean_operation(trimmed);
    }

    // Handle not operator
    if trimmed.starts_with('!') {
        let operand = parse_simple(&trimmed[1..])?;
        return Ok(EchoAst::Not {
            operand: Box::new(operand),
        });
    }

    // Handle binary operations
    if let Some(op) = find_binary_operator(trimmed) {
        return parse_binary_operation(trimmed, op);
    }

    // Handle literals and identifiers
    parse_primary(trimmed)
}

fn parse_let_statement(input: &str) -> Result<EchoAst, String> {
    let parts: Vec<&str> = input[4..].split('=').collect();
    if parts.len() != 2 {
        return Err("Invalid let statement".to_string());
    }

    let name = parts[0].trim().trim_end_matches(';').to_string();
    if name.is_empty() {
        return Err("Empty variable name in let statement".to_string());
    }

    let value_str = parts[1].trim().trim_end_matches(';');
    let value = parse_primary(value_str)?;

    Ok(EchoAst::Assignment {
        target: LValue::Binding {
            binding_type: BindingType::Let,
            pattern: BindingPattern::Identifier(name),
        },
        value: Box::new(value),
    })
}

fn parse_const_statement(input: &str) -> Result<EchoAst, String> {
    let parts: Vec<&str> = input[6..].split('=').collect();
    if parts.len() != 2 {
        return Err("Invalid const statement".to_string());
    }

    let name = parts[0].trim().trim_end_matches(';').to_string();
    let value_str = parts[1].trim().trim_end_matches(';');
    let value = parse_primary(value_str)?;

    Ok(EchoAst::Assignment {
        target: LValue::Binding {
            binding_type: BindingType::Const,
            pattern: BindingPattern::Identifier(name),
        },
        value: Box::new(value),
    })
}

fn parse_assignment(input: &str) -> Result<EchoAst, String> {
    let parts: Vec<&str> = input.split('=').collect();
    if parts.len() != 2 {
        return Err("Invalid assignment".to_string());
    }

    let left = parts[0].trim();
    let right = parts[1].trim().trim_end_matches(';');

    // Check if it's a property assignment
    if left.contains('.') {
        let dot_parts: Vec<&str> = left.split('.').collect();
        if dot_parts.len() == 2 {
            let obj_name = dot_parts[0].trim();
            let prop_name = dot_parts[1].trim();

            let object_ast = if obj_name.starts_with('$') {
                EchoAst::SystemProperty(obj_name[1..].to_string())
            } else {
                EchoAst::Identifier(obj_name.to_string())
            };

            let value_ast = parse_primary(right)?;

            return Ok(EchoAst::Assignment {
                target: LValue::PropertyAccess {
                    object: Box::new(object_ast),
                    property: prop_name.to_string(),
                },
                value: Box::new(value_ast),
            });
        }
    }

    // Simple identifier assignment
    if is_valid_identifier(left) {
        let value_ast = parse_primary(right)?;
        return Ok(EchoAst::Assignment {
            target: LValue::Binding {
                binding_type: BindingType::None,
                pattern: BindingPattern::Identifier(left.to_string()),
            },
            value: Box::new(value_ast),
        });
    }

    Err("Invalid assignment target".to_string())
}

fn parse_property_access(input: &str) -> Result<EchoAst, String> {
    let dot_pos = input.find('.').ok_or("No dot in property access")?;
    let obj_str = input[..dot_pos].trim();
    let prop_str = input[dot_pos + 1..].trim();

    if prop_str.is_empty() {
        return Err("Empty property name".to_string());
    }

    let object_ast = if obj_str.starts_with('$') {
        EchoAst::SystemProperty(obj_str[1..].to_string())
    } else {
        EchoAst::Identifier(obj_str.to_string())
    };

    Ok(EchoAst::PropertyAccess {
        object: Box::new(object_ast),
        property: prop_str.to_string(),
    })
}

fn parse_method_call(input: &str) -> Result<EchoAst, String> {
    let colon_pos = input.find(':').ok_or("No colon in method call")?;
    let obj_str = input[..colon_pos].trim();
    let rest = input[colon_pos + 1..].trim();

    let paren_start = rest.find('(').ok_or("No opening parenthesis")?;
    if !rest.ends_with(')') {
        return Err("Method call missing closing parenthesis".to_string());
    }

    let method_name = rest[..paren_start].trim().to_string();
    let args_str = &rest[paren_start + 1..rest.len() - 1];

    let args = if args_str.trim().is_empty() {
        vec![]
    } else {
        args_str
            .split(',')
            .map(|arg| parse_primary(arg.trim()))
            .collect::<Result<Vec<_>, _>>()?
    };

    Ok(EchoAst::MethodCall {
        object: Box::new(EchoAst::Identifier(obj_str.to_string())),
        method: method_name,
        args,
    })
}

fn parse_binary_with_property(input: &str) -> Result<EchoAst, String> {
    // Find the operator
    let op_char = input
        .chars()
        .find(|&c| "+-*/".contains(c))
        .ok_or("No operator found")?;

    let parts: Vec<&str> = input.split(op_char).collect();
    if parts.len() != 2 {
        return Err("Invalid binary operation".to_string());
    }

    let left = parse_property_access(parts[0].trim())?;
    let right = parse_primary(parts[1].trim())?;

    match op_char {
        '+' => Ok(EchoAst::Add {
            left: Box::new(left),
            right: Box::new(right),
        }),
        '-' => Ok(EchoAst::Subtract {
            left: Box::new(left),
            right: Box::new(right),
        }),
        '*' => Ok(EchoAst::Multiply {
            left: Box::new(left),
            right: Box::new(right),
        }),
        '/' => Ok(EchoAst::Divide {
            left: Box::new(left),
            right: Box::new(right),
        }),
        _ => Err(format!("Unsupported operator: {}", op_char)),
    }
}

fn parse_binary_operation(input: &str, op: char) -> Result<EchoAst, String> {
    let parts: Vec<&str> = input.split(op).collect();
    if parts.len() != 2 {
        return Err("Invalid binary operation".to_string());
    }

    let left = parse_primary(parts[0].trim())?;
    let right = parse_primary(parts[1].trim())?;

    match op {
        '+' => Ok(EchoAst::Add {
            left: Box::new(left),
            right: Box::new(right),
        }),
        '-' => Ok(EchoAst::Subtract {
            left: Box::new(left),
            right: Box::new(right),
        }),
        '*' => Ok(EchoAst::Multiply {
            left: Box::new(left),
            right: Box::new(right),
        }),
        '/' => Ok(EchoAst::Divide {
            left: Box::new(left),
            right: Box::new(right),
        }),
        _ => Err(format!("Unsupported operator: {}", op)),
    }
}

fn parse_list(input: &str) -> Result<EchoAst, String> {
    let content = &input[1..input.len() - 1];
    if content.trim().is_empty() {
        return Ok(EchoAst::List { elements: vec![] });
    }
    
    let elements = content
        .split(',')
        .map(|elem| parse_primary(elem.trim()))
        .collect::<Result<Vec<_>, _>>()?;
    
    Ok(EchoAst::List { elements })
}

fn parse_boolean_operation(input: &str) -> Result<EchoAst, String> {
    if let Some(pos) = input.find("&&") {
        let left = parse_simple(&input[..pos].trim())?;
        let right = parse_simple(&input[pos + 2..].trim())?;
        return Ok(EchoAst::And {
            left: Box::new(left),
            right: Box::new(right),
        });
    }
    
    if let Some(pos) = input.find("||") {
        let left = parse_simple(&input[..pos].trim())?;
        let right = parse_simple(&input[pos + 2..].trim())?;
        return Ok(EchoAst::Or {
            left: Box::new(left),
            right: Box::new(right),
        });
    }
    
    Err("Invalid boolean operation".to_string())
}

#[allow(clippy::excessive_nesting)]
fn parse_object_definition(input: &str) -> Result<EchoAst, String> {
    let lines: Vec<&str> = input
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect();

    if lines.len() < 2 || lines.last() != Some(&"endobject") {
        return Err("Invalid object definition".to_string());
    }

    let header_parts: Vec<&str> = lines[0].split_whitespace().collect();
    if header_parts.len() < 2 {
        return Err("Invalid object header".to_string());
    }

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
            let prop_def = &line[9..];
            let prop_parts: Vec<&str> = prop_def.split('=').collect();
            if prop_parts.len() >= 2 {
                let prop_name = prop_parts[0].trim().trim_end_matches(';').to_string();
                let prop_value = prop_parts[1].trim().trim_end_matches(';');
                let value_ast = parse_primary(prop_value)?;

                members.push(ObjectMember::Property {
                    name: prop_name,
                    value: value_ast,
                    permissions: None,
                });
            }
        } else if line.starts_with("verb ") {
            let verb_line = &line[5..];
            if let Some(paren_start) = verb_line.find('(') {
                let verb_name = verb_line[..paren_start]
                    .trim()
                    .trim_matches('"')
                    .to_string();

                // Collect verb body
                let mut verb_body = Vec::new();
                i += 1;
                while i < lines.len() - 1 && lines[i].trim() != "endverb" {
                    let line_content = lines[i].trim();
                    if !line_content.is_empty() {
                        // Handle return statements
                        if line_content.starts_with("return ") {
                            let return_expr = &line_content[7..].trim_end_matches(';');
                            if let Ok(expr) = parse_primary(return_expr) {
                                verb_body.push(EchoAst::Return {
                                    value: Some(Box::new(expr)),
                                });
                            }
                        } else if let Ok(stmt) = parse_simple(line_content) {
                            verb_body.push(stmt);
                        }
                    }
                    i += 1;
                }

                members.push(ObjectMember::Verb {
                    name: verb_name,
                    args: vec![], // Simple parser doesn't parse args
                    body: verb_body,
                    permissions: None,
                });
            }
        }
        i += 1;
    }

    Ok(EchoAst::ObjectDef {
        name,
        parent,
        members,
    })
}

fn parse_primary(input: &str) -> Result<EchoAst, String> {
    let trimmed = input.trim();

    // Numbers
    if let Ok(num) = trimmed.parse::<i64>() {
        return Ok(EchoAst::Number(num));
    }

    if let Ok(num) = trimmed.parse::<f64>() {
        return Ok(EchoAst::Float(num));
    }

    // Strings
    if trimmed.starts_with('"') && trimmed.ends_with('"') && trimmed.len() >= 2 {
        return Ok(EchoAst::String(trimmed[1..trimmed.len() - 1].to_string()));
    }

    // Booleans
    if trimmed == "true" {
        return Ok(EchoAst::Boolean(true));
    }
    if trimmed == "false" {
        return Ok(EchoAst::Boolean(false));
    }

    // Null
    if trimmed == "null" {
        return Ok(EchoAst::Null);
    }

    // System properties
    if trimmed.starts_with('$') {
        return Ok(EchoAst::SystemProperty(trimmed[1..].to_string()));
    }

    // Object references
    if trimmed.starts_with('#') && trimmed.len() > 1 {
        if let Ok(num) = trimmed[1..].parse::<i64>() {
            return Ok(EchoAst::ObjectRef(num));
        }
    }

    // Identifiers
    if is_valid_identifier(trimmed) {
        return Ok(EchoAst::Identifier(trimmed.to_string()));
    }

    Err(format!("Unable to parse: '{}'", trimmed))
}

fn find_binary_operator(input: &str) -> Option<char> {
    // Simple operator detection - doesn't handle precedence or nested expressions
    for (i, c) in input.char_indices() {
        if "+-*/".contains(c) {
            // Make sure it's not inside a string
            let before = &input[..i];
            if before.matches('"').count() % 2 == 0 {
                return Some(c);
            }
        }
    }
    None
}

fn is_valid_identifier(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    let mut chars = s.chars();
    let first = chars.next().unwrap();
    if !first.is_alphabetic() && first != '_' {
        return false;
    }
    chars.all(|c| c.is_alphanumeric() || c == '_')
}