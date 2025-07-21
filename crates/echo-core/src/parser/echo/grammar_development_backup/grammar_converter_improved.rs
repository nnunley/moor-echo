// Converter from improved grammar to unified AST
use anyhow::{Result, anyhow};
use crate::ast;
use super::grammar_improved as g;

/// Convert improved grammar Program to unified AST
pub fn convert_program(program: g::Program) -> Result<Vec<ast::EchoAst>> {
    program.statements.into_iter()
        .map(convert_statement)
        .collect::<Result<Vec<_>>>()
}

/// Convert Statement to unified AST
fn convert_statement(stmt: g::Statement) -> Result<ast::EchoAst> {
    match stmt {
        g::Statement::Expression(expr_stmt) => {
            convert_expression(*expr_stmt.expression)
        },
        g::Statement::Let(let_stmt) => {
            Ok(ast::EchoAst::LocalAssignment {
                target: convert_pattern(*let_stmt.pattern)?,
                value: Box::new(convert_expression(*let_stmt.expression)?),
            })
        },
        g::Statement::Const(const_stmt) => {
            Ok(ast::EchoAst::ConstAssignment {
                target: convert_pattern(*const_stmt.pattern)?,
                value: Box::new(convert_expression(*const_stmt.expression)?),
            })
        },
        g::Statement::Global(global_stmt) => {
            // Global assignments become regular assignments for now
            Ok(ast::EchoAst::Assignment {
                target: ast::LValue::Binding {
                    binding_type: ast::BindingType::None,
                    pattern: convert_pattern(*global_stmt.pattern)?,
                },
                value: Box::new(convert_expression(*global_stmt.expression)?),
            })
        },
        g::Statement::If(if_stmt) => {
            let then_branch = if_stmt.then_body.into_iter()
                .map(convert_statement)
                .collect::<Result<Vec<_>>>()?;
            
            let else_branch = if let Some(else_clause) = if_stmt.else_clause {
                Some(else_clause.body.into_iter()
                    .map(convert_statement)
                    .collect::<Result<Vec<_>>>()?)
            } else {
                None
            };
            
            Ok(ast::EchoAst::If {
                condition: Box::new(convert_expression(*if_stmt.condition)?),
                then_branch,
                else_branch,
            })
        },
        g::Statement::While(while_stmt) => {
            let body = while_stmt.body.into_iter()
                .map(convert_statement)
                .collect::<Result<Vec<_>>>()?;
            
            Ok(ast::EchoAst::While {
                label: while_stmt.label.map(|id| id.name),
                condition: Box::new(convert_expression(*while_stmt.condition)?),
                body,
            })
        },
        g::Statement::For(for_stmt) => {
            let variable = match convert_pattern(*for_stmt.pattern)? {
                ast::BindingPattern::Identifier(name) => name,
                _ => return Err(anyhow!("For loop variable must be simple identifier")),
            };
            
            let body = for_stmt.body.into_iter()
                .map(convert_statement)
                .collect::<Result<Vec<_>>>()?;
            
            Ok(ast::EchoAst::For {
                label: for_stmt.label.map(|id| id.name),
                variable,
                collection: Box::new(convert_expression(*for_stmt.iterable)?),
                body,
            })
        },
        g::Statement::Return(ret_stmt) => {
            Ok(ast::EchoAst::Return {
                value: if let Some(expr) = ret_stmt.value {
                    Some(Box::new(convert_expression(*expr)?))
                } else {
                    None
                },
            })
        },
        g::Statement::Break(break_stmt) => {
            Ok(ast::EchoAst::Break {
                label: break_stmt.label.map(|id| id.name),
            })
        },
        g::Statement::Continue(cont_stmt) => {
            Ok(ast::EchoAst::Continue {
                label: cont_stmt.label.map(|id| id.name),
            })
        },
        g::Statement::Block(block_stmt) => {
            let statements = block_stmt.statements.into_iter()
                .map(convert_statement)
                .collect::<Result<Vec<_>>>()?;
            Ok(ast::EchoAst::Block(statements))
        },
        _ => Err(anyhow!("Unsupported statement type")),
    }
}

/// Convert Expression to unified AST
fn convert_expression(expr: g::Expression) -> Result<ast::EchoAst> {
    match expr {
        // Literals
        g::Expression::Integer(lit) => Ok(ast::EchoAst::Number(lit.value)),
        g::Expression::Float(lit) => Ok(ast::EchoAst::Float(lit.value)),
        g::Expression::String(lit) => Ok(ast::EchoAst::String(lit.value)),
        g::Expression::True => Ok(ast::EchoAst::Boolean(true)),
        g::Expression::False => Ok(ast::EchoAst::Boolean(false)),
        g::Expression::Null => Ok(ast::EchoAst::Null),
        
        // Identifiers and references
        g::Expression::Identifier(id) => Ok(ast::EchoAst::Identifier(id.name)),
        g::Expression::ObjectRef(obj_ref) => Ok(ast::EchoAst::ObjectRef(obj_ref.value)),
        g::Expression::SystemProperty(sys_prop) => Ok(ast::EchoAst::SystemProperty(sys_prop.value)),
        
        // Collections
        g::Expression::List(list_lit) => {
            let elements = list_lit.elements.into_iter()
                .map(|elem| match elem {
                    g::ListElement::Expression(expr) => convert_expression(*expr),
                    g::ListElement::Scatter { expression, .. } => convert_expression(*expression),
                })
                .collect::<Result<Vec<_>>>()?;
            Ok(ast::EchoAst::List { elements })
        },
        
        // Binary operations
        g::Expression::Assignment { target, value, .. } => {
            let lvalue = convert_expression_to_lvalue(*target)?;
            Ok(ast::EchoAst::Assignment {
                target: lvalue,
                value: Box::new(convert_expression(*value)?),
            })
        },
        g::Expression::Or { left, right, .. } => {
            Ok(ast::EchoAst::Or {
                left: Box::new(convert_expression(*left)?),
                right: Box::new(convert_expression(*right)?),
            })
        },
        g::Expression::And { left, right, .. } => {
            Ok(ast::EchoAst::And {
                left: Box::new(convert_expression(*left)?),
                right: Box::new(convert_expression(*right)?),
            })
        },
        g::Expression::In { left, right, .. } => {
            Ok(ast::EchoAst::In {
                left: Box::new(convert_expression(*left)?),
                right: Box::new(convert_expression(*right)?),
            })
        },
        g::Expression::Equal { left, right, .. } => {
            Ok(ast::EchoAst::Equal {
                left: Box::new(convert_expression(*left)?),
                right: Box::new(convert_expression(*right)?),
            })
        },
        g::Expression::NotEqual { left, right, .. } => {
            Ok(ast::EchoAst::NotEqual {
                left: Box::new(convert_expression(*left)?),
                right: Box::new(convert_expression(*right)?),
            })
        },
        g::Expression::Less { left, right, .. } => {
            Ok(ast::EchoAst::LessThan {
                left: Box::new(convert_expression(*left)?),
                right: Box::new(convert_expression(*right)?),
            })
        },
        g::Expression::LessEqual { left, right, .. } => {
            Ok(ast::EchoAst::LessEqual {
                left: Box::new(convert_expression(*left)?),
                right: Box::new(convert_expression(*right)?),
            })
        },
        g::Expression::Greater { left, right, .. } => {
            Ok(ast::EchoAst::GreaterThan {
                left: Box::new(convert_expression(*left)?),
                right: Box::new(convert_expression(*right)?),
            })
        },
        g::Expression::GreaterEqual { left, right, .. } => {
            Ok(ast::EchoAst::GreaterEqual {
                left: Box::new(convert_expression(*left)?),
                right: Box::new(convert_expression(*right)?),
            })
        },
        g::Expression::Add { left, right, .. } => {
            Ok(ast::EchoAst::Add {
                left: Box::new(convert_expression(*left)?),
                right: Box::new(convert_expression(*right)?),
            })
        },
        g::Expression::Subtract { left, right, .. } => {
            Ok(ast::EchoAst::Subtract {
                left: Box::new(convert_expression(*left)?),
                right: Box::new(convert_expression(*right)?),
            })
        },
        g::Expression::Multiply { left, right, .. } => {
            Ok(ast::EchoAst::Multiply {
                left: Box::new(convert_expression(*left)?),
                right: Box::new(convert_expression(*right)?),
            })
        },
        g::Expression::Divide { left, right, .. } => {
            Ok(ast::EchoAst::Divide {
                left: Box::new(convert_expression(*left)?),
                right: Box::new(convert_expression(*right)?),
            })
        },
        g::Expression::Modulo { left, right, .. } => {
            Ok(ast::EchoAst::Modulo {
                left: Box::new(convert_expression(*left)?),
                right: Box::new(convert_expression(*right)?),
            })
        },
        g::Expression::Power { left, right, .. } => {
            Ok(ast::EchoAst::Power {
                left: Box::new(convert_expression(*left)?),
                right: Box::new(convert_expression(*right)?),
            })
        },
        
        // Unary operations
        g::Expression::Not { operand, .. } => {
            Ok(ast::EchoAst::Not {
                operand: Box::new(convert_expression(*operand)?),
            })
        },
        g::Expression::Negate { operand, .. } => {
            Ok(ast::EchoAst::UnaryMinus {
                operand: Box::new(convert_expression(*operand)?),
            })
        },
        
        // Access operations
        g::Expression::PropertyAccess { object, property, .. } => {
            Ok(ast::EchoAst::PropertyAccess {
                object: Box::new(convert_expression(*object)?),
                property: property.name,
            })
        },
        g::Expression::MethodCall { object, method, arguments, .. } => {
            let args = arguments.into_iter()
                .map(convert_expression)
                .collect::<Result<Vec<_>>>()?;
            Ok(ast::EchoAst::MethodCall {
                object: Box::new(convert_expression(*object)?),
                method: method.name,
                args,
            })
        },
        g::Expression::IndexAccess { object, index, .. } => {
            Ok(ast::EchoAst::IndexAccess {
                object: Box::new(convert_expression(*object)?),
                index: Box::new(convert_expression(*index)?),
            })
        },
        
        // Function calls
        g::Expression::Call { function, arguments, .. } => {
            let args = arguments.into_iter()
                .map(|arg| match arg {
                    g::CallArgument::Expression(expr) => convert_expression(*expr),
                    g::CallArgument::Scatter { expression, .. } => convert_expression(*expression),
                })
                .collect::<Result<Vec<_>>>()?;
            Ok(ast::EchoAst::Call {
                func: Box::new(convert_expression(*function)?),
                args,
            })
        },
        
        // Lambda expressions
        g::Expression::Lambda { parameters, body, .. } => {
            let params = convert_pattern_to_lambda_params(*parameters)?;
            let body_ast = match body {
                g::LambdaBody::Expression(expr) => convert_expression(*expr)?,
                g::LambdaBody::Block { statements, .. } => {
                    let stmts = statements.into_iter()
                        .map(convert_statement)
                        .collect::<Result<Vec<_>>>()?;
                    ast::EchoAst::Program(stmts)
                },
            };
            Ok(ast::EchoAst::Lambda {
                params,
                body: Box::new(body_ast),
            })
        },
        
        // List comprehensions
        g::Expression::ListComprehension { expression, pattern, iterable, condition, .. } => {
            // Convert to a function call that generates the list
            // This is a simplified conversion - full implementation would need runtime support
            let _expr = convert_expression(*expression)?;
            let _pat = convert_pattern(*pattern)?;
            let _iter = convert_expression(*iterable)?;
            let _cond = if let Some(c) = condition {
                Some(convert_expression(*c)?)
            } else {
                None
            };
            
            // For now, return an empty list as placeholder
            Ok(ast::EchoAst::List { elements: vec![] })
        },
        
        // Parenthesized expressions
        g::Expression::Parenthesized { expression, .. } => {
            convert_expression(*expression)
        },
        
        _ => Err(anyhow!("Unsupported expression type")),
    }
}

/// Convert Pattern to BindingPattern
fn convert_pattern(pattern: g::Pattern) -> Result<ast::BindingPattern> {
    match pattern {
        g::Pattern::Identifier(id) => Ok(ast::BindingPattern::Identifier(id.name)),
        g::Pattern::List { elements, .. } => {
            let converted_elements = elements.into_iter()
                .map(|elem| match elem {
                    g::PatternElement::Simple(id) => Ok(ast::BindingPatternElement::Simple(id.name)),
                    g::PatternElement::Optional { name, default, .. } => {
                        Ok(ast::BindingPatternElement::Optional {
                            name: name.name,
                            default: Box::new(convert_expression(*default)?),
                        })
                    },
                    g::PatternElement::Rest { name, .. } => {
                        Ok(ast::BindingPatternElement::Rest(name.name))
                    },
                })
                .collect::<Result<Vec<_>>>()?;
            Ok(ast::BindingPattern::List(converted_elements))
        },
        g::Pattern::Rest { name, .. } => {
            Ok(ast::BindingPattern::Rest(Box::new(ast::BindingPattern::Identifier(name.name))))
        },
        g::Pattern::Ignore => Ok(ast::BindingPattern::Ignore),
    }
}

/// Convert Pattern to LambdaParam vector
fn convert_pattern_to_lambda_params(pattern: g::Pattern) -> Result<Vec<ast::LambdaParam>> {
    match pattern {
        g::Pattern::Identifier(id) => Ok(vec![ast::LambdaParam::Simple(id.name)]),
        g::Pattern::List { elements, .. } => {
            elements.into_iter()
                .map(|elem| match elem {
                    g::PatternElement::Simple(id) => Ok(ast::LambdaParam::Simple(id.name)),
                    g::PatternElement::Optional { name, default, .. } => {
                        Ok(ast::LambdaParam::Optional {
                            name: name.name,
                            default: Box::new(convert_expression(*default)?),
                        })
                    },
                    g::PatternElement::Rest { name, .. } => {
                        Ok(ast::LambdaParam::Rest(name.name))
                    },
                })
                .collect()
        },
        g::Pattern::Rest { name, .. } => Ok(vec![ast::LambdaParam::Rest(name.name)]),
        g::Pattern::Ignore => Ok(vec![]), // Ignore patterns don't contribute parameters
    }
}

/// Convert Expression to LValue for assignment targets
fn convert_expression_to_lvalue(expr: g::Expression) -> Result<ast::LValue> {
    match expr {
        g::Expression::Identifier(id) => Ok(ast::LValue::Binding {
            binding_type: ast::BindingType::None,
            pattern: ast::BindingPattern::Identifier(id.name),
        }),
        g::Expression::PropertyAccess { object, property, .. } => {
            Ok(ast::LValue::PropertyAccess {
                object: Box::new(convert_expression(*object)?),
                property: property.name,
            })
        },
        g::Expression::IndexAccess { object, index, .. } => {
            Ok(ast::LValue::IndexAccess {
                object: Box::new(convert_expression(*object)?),
                index: Box::new(convert_expression(*index)?),
            })
        },
        _ => Err(anyhow!("Invalid assignment target - must be identifier, property access, or index access")),
    }
}