// Modern Echo parser implementation
pub mod grammar;

use anyhow::{Result, anyhow};
use crate::ast::{self, LValue};

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

// Helper function to extract parameter names from ParamPattern (simple version for arrow functions)
fn _extract_simple_params_from_pattern(pattern: grammar::ParamPattern) -> Vec<String> {
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

// Helper function to extract full parameter info from ParamPattern
fn extract_lambda_params_from_pattern(pattern: grammar::ParamPattern) -> Result<Vec<ast::LambdaParam>> {
    use grammar::{ParamPattern as P, ParamElement as E};
    
    match pattern {
        P::Single(elem) => match elem {
            E::Simple(ident) => Ok(vec![ast::LambdaParam::Simple(ident.name)]),
            E::Optional { name, default, .. } => {
                let default_ast = Box::new(convert_grammar_to_ast(*default)?);
                Ok(vec![ast::LambdaParam::Optional { name: name.name, default: default_ast }])
            },
            E::Rest { name, .. } => Ok(vec![ast::LambdaParam::Rest(name.name)]),
        },
        P::Multiple { params, .. } => {
            params.into_iter().map(|elem| match elem {
                E::Simple(ident) => Ok(ast::LambdaParam::Simple(ident.name)),
                E::Optional { name, default, .. } => {
                    let default_ast = Box::new(convert_grammar_to_ast(*default)?);
                    Ok(ast::LambdaParam::Optional { name: name.name, default: default_ast })
                },
                E::Rest { name, .. } => Ok(ast::LambdaParam::Rest(name.name)),
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
        // Parse using the grammar
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
        // Parse the entire source as a program
        match self.inner.parse(source) {
            Ok(ast) => {
                // If it's already a program node, convert it directly
                convert_grammar_to_ast(ast)
            }
            Err(_) => {
                // If parsing as a single unit fails, try parsing line by line
                let mut statements = Vec::new();
                let mut current_lines = Vec::new();
                
                for line in source.lines() {
                    let trimmed = line.trim();
                    
                    // Skip empty lines and comments
                    if trimmed.is_empty() || trimmed.starts_with("#") {
                        continue;
                    }
                    
                    current_lines.push(line);
                    let accumulated = current_lines.join("\n");
                    
                    // Try to parse accumulated lines
                    match self.parse(&accumulated) {
                        Ok(stmt) => {
                            statements.push(stmt);
                            current_lines.clear();
                        }
                        Err(_) => {
                            // Need more lines, continue accumulating
                        }
                    }
                }
                
                // If we have leftover lines, try to parse them
                if !current_lines.is_empty() {
                    let remaining = current_lines.join("\n");
                    match self.parse(&remaining) {
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
            let lvalue = match convert_grammar_to_ast(*target)? {
                ast::EchoAst::Identifier(name) => {
                    ast::LValue::Binding {
                        binding_type: ast::BindingType::None,
                        pattern: ast::BindingPattern::Identifier(name),
                    }
                }
                ast::EchoAst::PropertyAccess { object, property } => {
                    ast::LValue::PropertyAccess { object, property }
                }
                _ => return Err(anyhow!("Assignment target must be an identifier or property access")),
            };
            
            ast::EchoAst::Assignment {
                target: lvalue,
                value: Box::new(convert_grammar_to_ast(*value)?),
            }
        },
        
        G::LetBinding { pattern, value, .. } => {
            let binding_pattern = match convert_grammar_to_ast(*pattern)? {
                ast::EchoAst::Identifier(name) => ast::BindingPattern::Identifier(name),
                ast::EchoAst::List { elements } => {
                    // Convert list elements to binding pattern
                    let patterns = elements.into_iter().map(|elem| {
                        match elem {
                            ast::EchoAst::Identifier(name) => Ok(ast::BindingPattern::Identifier(name)),
                            _ => Err(anyhow!("Invalid pattern in let binding")),
                        }
                    }).collect::<Result<Vec<_>>>()?;
                    ast::BindingPattern::List(patterns)
                }
                _ => return Err(anyhow!("Invalid pattern in let binding")),
            };
            
            ast::EchoAst::Assignment {
                target: ast::LValue::Binding {
                    binding_type: ast::BindingType::Let,
                    pattern: binding_pattern,
                },
                value: Box::new(convert_grammar_to_ast(*value)?),
            }
        },
        
        G::ConstBinding { pattern, value, .. } => {
            let binding_pattern = match convert_grammar_to_ast(*pattern)? {
                ast::EchoAst::Identifier(name) => ast::BindingPattern::Identifier(name),
                ast::EchoAst::List { elements } => {
                    // Convert list elements to binding pattern
                    let patterns = elements.into_iter().map(|elem| {
                        match elem {
                            ast::EchoAst::Identifier(name) => Ok(ast::BindingPattern::Identifier(name)),
                            _ => Err(anyhow!("Invalid pattern in const binding")),
                        }
                    }).collect::<Result<Vec<_>>>()?;
                    ast::BindingPattern::List(patterns)
                }
                _ => return Err(anyhow!("Invalid pattern in const binding")),
            };
            
            ast::EchoAst::Assignment {
                target: ast::LValue::Binding {
                    binding_type: ast::BindingType::Const,
                    pattern: binding_pattern,
                },
                value: Box::new(convert_grammar_to_ast(*value)?),
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
            // For arrow functions, we need to check if it's a simple identifier or a pattern
            let lambda_params = match params.as_ref() {
                // Single identifier like: x => ...
                G::Identifier(name) => vec![ast::LambdaParam::Simple(name.clone())],
                // Pattern like: {x, y} => ... or {x, ?y=5} => ...
                _ => {
                    // Try to parse as a pattern from the params expression
                    // For now, just extract simple names until we have better pattern parsing
                    extract_params_from_scatter(params)?
                        .into_iter()
                        .map(ast::LambdaParam::Simple)
                        .collect()
                }
            };
            A::Lambda {
                params: lambda_params,
                body: Box::new(convert_grammar_to_ast(*body)?),
            }
        },
        
        G::BlockFunction { params, body, .. } => {
            // Extract full parameter info from the parameter pattern
            let lambda_params = extract_lambda_params_from_pattern(params)?;
            // Convert body to a single expression (Program)
            let body_stmts = body.into_iter()
                .map(convert_grammar_to_ast)
                .collect::<Result<Vec<_>>>()?;
            A::Lambda {
                params: lambda_params,
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


// Re-export for backward compatibility
pub use grammar::parse_echo;