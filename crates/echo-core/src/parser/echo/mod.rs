// Modern Echo parser implementation
pub mod grammar;
// The following modules were moved to grammar_development_backup/ for future
// reference pub mod grammar_improved;
// pub mod grammar_converter_improved;
// pub mod parser_improved;
// #[cfg(test)]
// pub mod test_improved_grammar;

#[cfg(test)]
mod match_try_tests;

use anyhow::{anyhow, Result};

use crate::ast::{self};

// Helper function to convert match patterns
fn convert_match_pattern(pattern: grammar::MatchPattern) -> Result<ast::Pattern> {
    use grammar::MatchPattern as G;
    use crate::ast::Pattern as A;
    
    match pattern {
        G::Wildcard => Ok(A::Wildcard),
        G::Number(n) => Ok(A::Number(n)),
        G::String(s) => Ok(A::String(s)),
        G::Identifier(id) => Ok(A::Identifier(id.name)),
    }
}

// Helper function to extract lambda parameters from scatter expressions
fn extract_lambda_params_from_scatter(scatter: grammar::EchoAst) -> Result<Vec<ast::LambdaParam>> {
    use grammar::EchoAst as G;

    match scatter {
        // Simple identifier: x
        G::Identifier(name) => Ok(vec![ast::LambdaParam::Simple(name)]),

        // Brace expression: {x, y, z} - only simple identifiers for now
        G::BraceExpression { elements, .. } => {
            let mut params = Vec::new();
            for elem in elements {
                match elem {
                    G::Identifier(name) => params.push(ast::LambdaParam::Simple(name)),
                    _ => {
                        return Err(anyhow!(
                            "Only simple identifiers supported in lambda parameters for now"
                        ))
                    }
                }
            }
            Ok(params)
        }

        // Parenthesized expression: (x) or (x, y)
        G::Paren { expr, .. } => extract_lambda_params_from_scatter(*expr),

        _ => Err(anyhow!("Invalid parameter pattern")),
    }
}

// Helper function to extract parameter names from ParamPattern (simple version
// for arrow functions)
fn _extract_simple_params_from_pattern(pattern: grammar::ParamPattern) -> Vec<String> {
    use grammar::{ParamElement as E, ParamPattern as P};

    match pattern {
        P::Single(elem) => match elem {
            E::Simple(ident) => vec![ident.name],
            E::Optional { name, .. } => vec![name.name],
            E::Rest { name, .. } => vec![name.name],
        },
        P::Multiple { params, .. } => params
            .into_iter()
            .map(|elem| match elem {
                E::Simple(ident) => ident.name,
                E::Optional { name, .. } => name.name,
                E::Rest { name, .. } => name.name,
            })
            .collect(),
    }
}

// Helper function to extract full parameter info from ParamPattern
fn extract_lambda_params_from_pattern(
    pattern: grammar::ParamPattern,
) -> Result<Vec<ast::LambdaParam>> {
    use grammar::{ParamElement as E, ParamPattern as P};

    match pattern {
        P::Single(elem) => match elem {
            E::Simple(ident) => Ok(vec![ast::LambdaParam::Simple(ident.name)]),
            E::Optional { name, default, .. } => {
                let default_ast = Box::new(convert_grammar_to_ast(*default)?);
                Ok(vec![ast::LambdaParam::Optional {
                    name: name.name,
                    default: default_ast,
                }])
            }
            E::Rest { name, .. } => Ok(vec![ast::LambdaParam::Rest(name.name)]),
        },
        P::Multiple { params, .. } => params
            .into_iter()
            .map(|elem| match elem {
                E::Simple(ident) => Ok(ast::LambdaParam::Simple(ident.name)),
                E::Optional { name, default, .. } => {
                    let default_ast = Box::new(convert_grammar_to_ast(*default)?);
                    Ok(ast::LambdaParam::Optional {
                        name: name.name,
                        default: default_ast,
                    })
                }
                E::Rest { name, .. } => Ok(ast::LambdaParam::Rest(name.name)),
            })
            .collect(),
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
        // Parse normally using the rust-sitter generated parser
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
    use ast::EchoAst as A;
    use grammar::EchoAst as G;

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

        G::Power { left, right, .. } => A::Power {
            left: Box::new(convert_grammar_to_ast(*left)?),
            right: Box::new(convert_grammar_to_ast(*right)?),
        },

        // Handle the generic Assignment Expression (renamed from Assignment)
        G::AssignmentExpr { left, right, .. } => {
            // Validate that left side is a valid LValue
            let lvalue = match convert_grammar_to_ast(*left)? {
                A::Identifier(name) => ast::LValue::Binding {
                    binding_type: ast::BindingType::None,
                    pattern: ast::BindingPattern::Identifier(name),
                },
                A::PropertyAccess { object, property } => {
                    ast::LValue::PropertyAccess { object, property }
                }
                A::IndexAccess { object, index } => ast::LValue::IndexAccess { object, index },
                _ => {
                    return Err(anyhow!(
                        "Invalid assignment target - must be identifier, property access, or \
                         index access"
                    ))
                }
            };

            A::Assignment {
                target: lvalue,
                value: Box::new(convert_grammar_to_ast(*right)?),
            }
        }

        // Handle the new LocalAssignment statement
        G::LocalAssignment { target, value, .. } => {
            let pattern = convert_grammar_binding_pattern_to_ast(*target)?;
            A::Assignment {
                target: ast::LValue::Binding {
                    binding_type: ast::BindingType::Let,
                    pattern,
                },
                value: Box::new(convert_grammar_to_ast(*value)?),
            }
        }

        // Handle the new ConstAssignment statement
        G::ConstAssignment { target, value, .. } => {
            let pattern = convert_grammar_binding_pattern_to_ast(*target)?;
            A::Assignment {
                target: ast::LValue::Binding {
                    binding_type: ast::BindingType::Const,
                    pattern,
                },
                value: Box::new(convert_grammar_to_ast(*value)?),
            }
        }

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

        // TODO: Re-enable In operator once conflict is resolved
        // G::In { left, right, .. } => A::In {
        //     left: Box::new(convert_grammar_to_ast(*left)?),
        //     right: Box::new(convert_grammar_to_ast(*right)?),
        // },
        G::And { left, right, .. } => A::And {
            left: Box::new(convert_grammar_to_ast(*left)?),
            right: Box::new(convert_grammar_to_ast(*right)?),
        },

        G::Or { left, right, .. } => A::Or {
            left: Box::new(convert_grammar_to_ast(*left)?),
            right: Box::new(convert_grammar_to_ast(*right)?),
        },

        G::UnaryMinus { operand, .. } => match operand.as_ref() {
            G::Number(n) => A::Number(-n),
            G::Float(f) => A::Float(-f),
            _ => A::Subtract {
                left: Box::new(A::Number(0)),
                right: Box::new(convert_grammar_to_ast(*operand)?),
            },
        },

        G::UnaryPlus { operand, .. } => {
            // Unary plus doesn't change the value, just return the operand
            convert_grammar_to_ast(*operand)?
        }

        G::Not { operand, .. } => A::Not {
            operand: Box::new(convert_grammar_to_ast(*operand)?),
        },

        G::PropertyAccess {
            object, property, ..
        } => A::PropertyAccess {
            object: Box::new(convert_grammar_to_ast(*object)?),
            property: property.name,
        },

        G::IndexAccess { object, index, .. } => A::IndexAccess {
            object: Box::new(convert_grammar_to_ast(*object)?),
            index: Box::new(convert_grammar_to_ast(*index)?),
        },

        G::MethodCall {
            object,
            method,
            args,
            ..
        } => {
            let method_name = method.name;
            let converted_args = args
                .into_iter()
                .filter(|arg| !matches!(arg, G::Comma))
                .map(convert_grammar_to_ast)
                .collect::<Result<Vec<_>>>()?;
            A::MethodCall {
                object: Box::new(convert_grammar_to_ast(*object)?),
                method: method_name,
                args: converted_args,
            }
        }

        G::List { elements, .. } => {
            let converted_elements = elements
                .into_iter()
                .filter(|elem| !matches!(elem, G::Comma))
                .map(convert_grammar_to_ast)
                .collect::<Result<Vec<_>>>()?;
            A::List {
                elements: converted_elements,
            }
        }

        G::ArrowFunction { params, body, .. } => {
            // For arrow functions, we need to check if it's a simple identifier or a
            // pattern
            let lambda_params = extract_lambda_params_from_scatter(*params)?;
            A::Lambda {
                params: lambda_params,
                body: Box::new(convert_grammar_to_ast(*body)?),
            }
        }

        G::BlockFunction { params, body, .. } => {
            // Extract full parameter info from the parameter pattern
            let lambda_params = extract_lambda_params_from_pattern(params)?;
            // Convert body to a single expression (Program)
            let body_stmts = body
                .into_iter()
                .map(convert_grammar_to_ast)
                .collect::<Result<Vec<_>>>()?;
            A::Lambda {
                params: lambda_params,
                body: Box::new(A::Program(body_stmts)),
            }
        }

        G::Call { func, args, .. } => {
            let converted_args = args
                .into_iter()
                .map(convert_grammar_to_ast)
                .collect::<Result<Vec<_>>>()?;

            // If func is an identifier, create FunctionCall; otherwise create Call
            match *func {
                G::Identifier(name) => A::FunctionCall {
                    name,
                    args: converted_args,
                },
                _ => {
                    let converted_func = Box::new(convert_grammar_to_ast(*func)?);
                    A::Call {
                        func: converted_func,
                        args: converted_args,
                    }
                }
            }
        }

        G::Paren { expr, .. } => {
            // Parentheses are just for grouping, return inner expression
            convert_grammar_to_ast(*expr)?
        }

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
                    grammar::ObjectMember::VerbDef {
                        name, params, requires_clause, body, ..
                    } => {
                        let verb_name = name.name.clone();
                        let verb_params = convert_params_to_parameters(&params)?;
                        let verb_body = body
                            .into_iter()
                            .map(convert_grammar_to_ast)
                            .collect::<Result<Vec<_>>>()?;
                        
                        // Extract required capabilities from requires clause
                        let required_capabilities = if let Some(clause) = requires_clause {
                            vec![clause.capability.name.name]
                        } else {
                            Vec::new()
                        };
                        
                        converted_members.push(ast::ObjectMember::Verb {
                            name: verb_name,
                            args: verb_params,
                            body: verb_body,
                            permissions: None,
                            required_capabilities,
                        });
                    }
                    grammar::ObjectMember::EventDef {
                        name, params, body, ..
                    } => {
                        let event_name = name.name.clone();
                        let event_params = convert_params_to_parameters(&params)?;
                        let event_body = body
                            .into_iter()
                            .map(convert_grammar_to_ast)
                            .collect::<Result<Vec<_>>>()?;
                        converted_members.push(ast::ObjectMember::Event {
                            name: event_name,
                            params: event_params,
                            body: event_body,
                        });
                    }
                    grammar::ObjectMember::QueryDef {
                        name,
                        params,
                        clauses,
                        ..
                    } => {
                        let query_name = name.name.clone();
                        let query_params = if let Some(p) = params {
                            p.params
                                .into_iter()
                                .filter_map(|param| match param {
                                    grammar::QueryParam::Identifier(id) => Some(id.name),
                                    _ => None, // Only identifiers allowed as query params
                                })
                                .collect()
                        } else {
                            Vec::new()
                        };

                        let query_clauses = clauses
                            .into_iter()
                            .map(convert_query_clause)
                            .collect::<Result<Vec<_>>>()?;

                        converted_members.push(ast::ObjectMember::Query {
                            name: query_name,
                            params: query_params,
                            clauses: query_clauses,
                        });
                    }
                }
            }

            A::ObjectDef {
                name: obj_name,
                parent: None,
                members: converted_members,
            }
        }

        G::Comma => {
            // Skip commas - they're just separators
            anyhow::bail!("Unexpected comma in AST conversion")
        }

        G::If {
            condition,
            then_body,
            else_clause,
            ..
        } => {
            // If with optional else clause using MOO syntax
            let then_vec = then_body
                .into_iter()
                .map(convert_grammar_to_ast)
                .collect::<Result<Vec<_>>>()?;
            let else_vec = if let Some(else_clause) = else_clause {
                Some(
                    else_clause
                        .body
                        .into_iter()
                        .map(convert_grammar_to_ast)
                        .collect::<Result<Vec<_>>>()?,
                )
            } else {
                None
            };

            A::If {
                condition: Box::new(convert_grammar_to_ast(*condition)?),
                then_branch: then_vec,
                else_branch: else_vec,
            }
        }

        G::While {
            condition, body, ..
        } => {
            let body_vec = body
                .into_iter()
                .map(convert_grammar_to_ast)
                .collect::<Result<Vec<_>>>()?;

            A::While {
                label: None, // Labels will be added later as separate feature
                condition: Box::new(convert_grammar_to_ast(*condition)?),
                body: body_vec,
            }
        }

        G::For {
            variable,
            collection,
            body,
            ..
        } => {
            let var_name = match variable.as_ref() {
                G::Identifier(s) => s.clone(),
                _ => anyhow::bail!("For loop variable must be identifier"),
            };

            let body_vec = body
                .into_iter()
                .map(convert_grammar_to_ast)
                .collect::<Result<Vec<_>>>()?;

            A::For {
                label: None, // For loops don't have labels in our grammar yet
                variable: var_name,
                collection: Box::new(convert_grammar_to_ast(*collection)?),
                body: body_vec,
            }
        }

        G::Match { expr, arms, .. } => {
            let converted_arms = arms
                .into_iter()
                .map(|arm| {
                    Ok(ast::MatchArm {
                        pattern: convert_match_pattern(arm.pattern)?,
                        guard: match arm.guard {
                            Some(guard) => Some(Box::new(convert_grammar_to_ast(*guard.condition)?)),
                            None => None,
                        },
                        body: Box::new(convert_grammar_to_ast(*arm.body)?),
                    })
                })
                .collect::<Result<Vec<_>>>()?;

            A::Match {
                expr: Box::new(convert_grammar_to_ast(*expr)?),
                arms: converted_arms,
            }
        }

        G::Try { body, catch, finally, .. } => {
            let try_body = body
                .into_iter()
                .map(convert_grammar_to_ast)
                .collect::<Result<Vec<_>>>()?;

            let catch_clause = match catch {
                Some(catch) => Some(ast::CatchClause {
                    error_var: Some(catch.error_var.name),
                    body: catch.body
                        .into_iter()
                        .map(convert_grammar_to_ast)
                        .collect::<Result<Vec<_>>>()?,
                }),
                None => None,
            };

            let finally_body = match finally {
                Some(finally) => Some(finally.body
                    .into_iter()
                    .map(convert_grammar_to_ast)
                    .collect::<Result<Vec<_>>>()?),
                None => None,
            };

            A::Try {
                body: try_body,
                catch: catch_clause,
                finally: finally_body,
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

        G::Return { value, .. } => {
            let return_val = if let Some(val) = value {
                Some(Box::new(convert_grammar_to_ast(*val)?))
            } else {
                None
            };

            A::Return { value: return_val }
        }

        G::Emit {
            event_name, args, ..
        } => {
            let emit_args = if let Some(emit_args) = args {
                emit_args
                    .args
                    .into_iter()
                    .filter(|arg| !matches!(arg, G::Comma))
                    .map(convert_grammar_to_ast)
                    .collect::<Result<Vec<_>>>()?
            } else {
                Vec::new()
            };

            A::Emit {
                event_name: event_name.name,
                args: emit_args,
            }
        }

        G::BraceExpression { .. } => {
            // BraceExpression should not appear as a standalone expression in the AST
            // It's only valid as lambda parameters
            return Err(anyhow!(
                "Brace expression can only be used as lambda parameters"
            ));
        }
    })
}

// Helper function for BindingPattern conversion
fn convert_grammar_binding_pattern_to_ast(
    node: grammar::BindingPattern,
) -> Result<ast::BindingPattern> {
    use ast::BindingPattern as A_BP;
    use grammar::BindingPattern as G_BP;

    Ok(match node {
        G_BP::Identifier(ident) => A_BP::Identifier(ident.name),
        G_BP::List { elements, .. } => {
            let converted_elements: Result<Vec<_>> = elements
                .into_iter()
                .map(convert_grammar_binding_pattern_element_to_ast)
                .collect();
            A_BP::List(converted_elements?)
        }
        G_BP::Rest { name, .. } => A_BP::Rest(Box::new(A_BP::Identifier(name.name))),
        G_BP::Ignore => A_BP::Ignore,
    })
}

// Helper function for BindingPatternElement conversion
fn convert_grammar_binding_pattern_element_to_ast(
    node: grammar::BindingPatternElement,
) -> Result<ast::BindingPatternElement> {
    use ast::BindingPatternElement as A_BPE;
    use grammar::BindingPatternElement as G_BPE;

    Ok(match node {
        G_BPE::Simple(ident) => A_BPE::Simple(ident.name),
        G_BPE::Optional { name, default, .. } => A_BPE::Optional {
            name: name.name,
            default: Box::new(convert_grammar_to_ast(*default)?),
        },
        G_BPE::Rest { name, .. } => A_BPE::Rest(name.name),
    })
}

// Helper function to convert ParamPattern to Parameter
fn convert_params_to_parameters(params: &grammar::ParamPattern) -> Result<Vec<ast::Parameter>> {
    use grammar::{ParamElement as G_PE, ParamPattern as G_PP};

    let elements = match params {
        G_PP::Single(elem) => vec![elem.clone()],
        G_PP::Multiple { params, .. } => params.to_vec(),
    };

    elements
        .into_iter()
        .map(|elem| {
            Ok(match elem {
                G_PE::Simple(ident) => ast::Parameter {
                    name: ident.name,
                    type_annotation: None,
                    default_value: None,
                },
                G_PE::Optional { name, default, .. } => ast::Parameter {
                    name: name.name,
                    type_annotation: None,
                    default_value: Some(convert_grammar_to_ast(*default)?),
                },
                G_PE::Rest { name, .. } => ast::Parameter {
                    name: format!("@{}", name.name), // Mark rest params with @
                    type_annotation: None,
                    default_value: None,
                },
            })
        })
        .collect()
}

// Helper function to convert query clause from grammar to AST
fn convert_query_clause(clause: grammar::QueryClause) -> Result<ast::QueryClause> {
    let predicate = clause.predicate.name;
    let args = clause
        .args
        .into_iter()
        .map(|arg| match arg {
            grammar::QueryParam::Identifier(id) => Ok(ast::QueryArg::Variable(id.name)),
            grammar::QueryParam::Number(n) => Ok(ast::QueryArg::Constant(ast::EchoAst::Number(n))),
            grammar::QueryParam::String(s) => Ok(ast::QueryArg::Constant(ast::EchoAst::String(s))),
            grammar::QueryParam::Wildcard { .. } => Ok(ast::QueryArg::Wildcard),
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(ast::QueryClause { predicate, args })
}

// Re-export for backward compatibility
pub use grammar::parse_echo;
