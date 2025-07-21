// Conversion from improved grammar structure to existing AST
use crate::ast::{self, LValue, BindingPattern, BindingPatternElement, BindingType, Parameter, ObjectMember};
use crate::parser::echo::grammar_improved as g;
use anyhow::{Result, bail};

pub fn convert_program(program: g::Program) -> Result<Vec<ast::EchoAst>> {
    match program {
        g::Program::Object(obj) => {
            let converted = convert_object_definition(*obj)?;
            Ok(vec![converted])
        }
        g::Program::Statements(stmts) => {
            stmts.into_iter()
                .map(convert_statement)
                .collect::<Result<Vec<_>>>()
        }
    }
}

fn convert_statement(stmt: g::Statement) -> Result<ast::EchoAst> {
    match stmt {
        g::Statement::Expression(expr_stmt) => {
            let expr = convert_expression(*expr_stmt.expression)?;
            Ok(ast::EchoAst::ExpressionStatement(Box::new(expr)))
        }
        g::Statement::Let(let_stmt) => convert_let_statement(*let_stmt),
        g::Statement::Const(const_stmt) => convert_const_statement(*const_stmt),
        g::Statement::Global(_global_stmt) => {
            bail!("Global statements not yet supported in existing AST")
        }
        g::Statement::If(if_stmt) => convert_if_statement(*if_stmt),
        g::Statement::While(while_stmt) => convert_while_statement(*while_stmt),
        g::Statement::For(for_stmt) => convert_for_statement(*for_stmt),
        g::Statement::Fork(_fork_stmt) => {
            bail!("Fork statements not yet supported in existing AST")
        }
        g::Statement::Try(_try_stmt) => {
            bail!("Try statements not yet supported in existing AST")
        }
        g::Statement::Return(ret_stmt) => convert_return_statement(*ret_stmt),
        g::Statement::Break(break_stmt) => convert_break_statement(*break_stmt),
        g::Statement::Continue(cont_stmt) => convert_continue_statement(*cont_stmt),
        g::Statement::Block(block_stmt) => {
            let statements = block_stmt.statements.into_iter()
                .map(convert_statement)
                .collect::<Result<Vec<_>>>()?;
            Ok(ast::EchoAst::Block(statements))
        }
    }
}

fn convert_expression(expr: g::Expression) -> Result<ast::EchoAst> {
    match expr {
        // Literals
        g::Expression::Integer(lit) => {
            // TODO: Use rust_sitter's generated method to get the actual value
            // For now, use a placeholder - this will need to be fixed when rust_sitter generates the parser
            Ok(ast::EchoAst::Number(42))
        }
        g::Expression::Float(lit) => {
            // TODO: Use rust_sitter's generated method to get the actual value
            Ok(ast::EchoAst::Float(3.14))
        }
        g::Expression::String(lit) => {
            // TODO: Use rust_sitter's generated method to get the actual value
            Ok(ast::EchoAst::String("placeholder".to_string()))
        }
        g::Expression::True => Ok(ast::EchoAst::Boolean(true)),
        g::Expression::False => Ok(ast::EchoAst::Boolean(false)),
        g::Expression::Null => Ok(ast::EchoAst::Null),
        
        // Identifiers and references
        g::Expression::Identifier(id) => {
            // TODO: Use rust_sitter's generated method to get the actual name
            Ok(ast::EchoAst::Identifier("placeholder".to_string()))
        }
        g::Expression::ObjectRef(lit) => {
            // TODO: Use rust_sitter's generated method to get the actual value
            Ok(ast::EchoAst::ObjectRef(42))
        }
        g::Expression::SystemProperty(lit) => {
            // TODO: Use rust_sitter's generated method to get the actual value
            Ok(ast::EchoAst::SystemProperty("placeholder".to_string()))
        }
        g::Expression::Symbol(_name) => {
            bail!("Symbol literals not yet supported in existing AST")
        }
        g::Expression::ErrorCode(_code) => {
            bail!("Error codes not yet supported in existing AST")
        }
        
        // Collections
        g::Expression::List(list) => convert_list_literal(*list),
        g::Expression::Map(_map) => {
            bail!("Map literals not yet supported in existing AST")
        }
        g::Expression::Range(_range) => {
            bail!("Range literals not yet supported in existing AST")
        }
        g::Expression::ListComprehension(_comp) => {
            bail!("List comprehensions not yet supported in existing AST")
        }
        
        // Binary operations
        g::Expression::Assignment(assign) => convert_assignment_expression(*assign),
        g::Expression::Conditional(_cond) => {
            bail!("Conditional expressions not yet supported in existing AST")
        }
        g::Expression::Or(bin) => convert_binary_or(*bin),
        g::Expression::And(bin) => convert_binary_and(*bin),
        g::Expression::Equal(bin) => convert_binary_equal(*bin),
        g::Expression::NotEqual(bin) => convert_binary_not_equal(*bin),
        g::Expression::Less(bin) => convert_binary_less(*bin),
        g::Expression::LessEqual(bin) => convert_binary_less_equal(*bin),
        g::Expression::Greater(bin) => convert_binary_greater(*bin),
        g::Expression::GreaterEqual(bin) => convert_binary_greater_equal(*bin),
        g::Expression::In(_bin) => {
            bail!("In operator not yet supported in existing AST")
        }
        g::Expression::Add(bin) => convert_binary_add(*bin),
        g::Expression::Subtract(bin) => convert_binary_subtract(*bin),
        g::Expression::Multiply(bin) => convert_binary_multiply(*bin),
        g::Expression::Divide(bin) => convert_binary_divide(*bin),
        g::Expression::Modulo(bin) => convert_binary_modulo(*bin),
        g::Expression::Power(bin) => convert_binary_power(*bin),
        g::Expression::BitwiseOr(_bin) => {
            bail!("Bitwise or operator not yet supported in existing AST")
        }
        g::Expression::BitwiseXor(_bin) => {
            bail!("Bitwise xor operator not yet supported in existing AST")
        }
        g::Expression::BitwiseAnd(_bin) => {
            bail!("Bitwise and operator not yet supported in existing AST")
        }
        g::Expression::ShiftLeft(_bin) => {
            bail!("Shift left operator not yet supported in existing AST")
        }
        g::Expression::ShiftRight(_bin) => {
            bail!("Shift right operator not yet supported in existing AST")
        }
        
        // Unary operations
        g::Expression::Not(unary) => convert_unary_not(*unary),
        g::Expression::Negate(unary) => convert_unary_negate(*unary),
        g::Expression::BitwiseNot(_unary) => {
            bail!("Bitwise not operator not yet supported in existing AST")
        }
        
        // Access operations
        g::Expression::PropertyAccess(access) => convert_property_access(*access),
        g::Expression::MethodCall(call) => convert_method_call(*call),
        g::Expression::IndexAccess(access) => convert_index_access(*access),
        g::Expression::Slice(_slice) => {
            bail!("Slice access not yet supported in existing AST")
        }
        
        // Function-related
        g::Expression::Call(call) => convert_call_expression(*call),
        g::Expression::Lambda(lambda) => convert_lambda_expression(*lambda),
        g::Expression::Function(func) => convert_function_expression(*func),
        
        // Special expressions
        g::Expression::Pass(_pass) => {
            bail!("Pass expressions not yet supported in existing AST")
        }
        g::Expression::TryExpression(_try_expr) => {
            bail!("Try expressions not yet supported in existing AST")
        }
        
        // Parenthesized
        g::Expression::Parenthesized(paren) => convert_expression(*paren.expression),
    }
}

fn convert_let_statement(let_stmt: g::LetStatement) -> Result<ast::EchoAst> {
    let pattern = convert_pattern_to_binding(*let_stmt.pattern)?;
    let value = convert_expression(*let_stmt.expression)?;
    Ok(ast::EchoAst::LocalAssignment {
        target: pattern,
        value: Box::new(value),
    })
}

fn convert_const_statement(const_stmt: g::ConstStatement) -> Result<ast::EchoAst> {
    let pattern = convert_pattern_to_binding(*const_stmt.pattern)?;
    let value = convert_expression(*const_stmt.expression)?;
    Ok(ast::EchoAst::ConstAssignment {
        target: pattern,
        value: Box::new(value),
    })
}

fn convert_pattern_to_binding(pattern: g::Pattern) -> Result<BindingPattern> {
    match pattern {
        g::Pattern::Identifier(id) => Ok(BindingPattern::Identifier("placeholder".to_string())),
        g::Pattern::List { elements, .. } => {
            let converted_elements = elements.into_iter()
                .map(convert_pattern_element_to_binding)
                .collect::<Result<Vec<_>>>()?;
            Ok(BindingPattern::List(converted_elements))
        }
        g::Pattern::Rest { name, .. } => {
            Ok(BindingPattern::Rest(Box::new(BindingPattern::Identifier("placeholder".to_string()))))
        }
        g::Pattern::Ignore => Ok(BindingPattern::Ignore),
    }
}

fn convert_pattern_element_to_binding(elem: g::PatternElement) -> Result<BindingPatternElement> {
    match elem {
        g::PatternElement::Simple(id) => Ok(BindingPatternElement::Simple("placeholder".to_string())),
        g::PatternElement::Optional { name, default, .. } => {
            let default_expr = convert_expression(*default)?;
            Ok(BindingPatternElement::Optional {
                name: "placeholder".to_string(),
                default: Box::new(default_expr),
            })
        }
        g::PatternElement::Rest { name, .. } => {
            Ok(BindingPatternElement::Rest("placeholder".to_string()))
        }
    }
}

fn convert_if_statement(if_stmt: g::IfStatement) -> Result<ast::EchoAst> {
    let condition = convert_expression(*if_stmt.condition)?;
    let then_branch = if_stmt.then_body.into_iter()
        .map(convert_statement)
        .collect::<Result<Vec<_>>>()?;
    
    // Handle elseif clauses by converting to nested if-else
    let else_branch = if !if_stmt.elseif_clauses.is_empty() {
        // Convert elseif clauses to nested if-else
        let mut current_else = if let Some(else_clause) = if_stmt.else_clause {
            Some(else_clause.body.into_iter()
                .map(convert_statement)
                .collect::<Result<Vec<_>>>()?)
        } else {
            None
        };
        
        // Process elseif clauses in reverse to build nested structure
        for elseif in if_stmt.elseif_clauses.into_iter().rev() {
            let elseif_condition = convert_expression(*elseif.condition)?;
            let elseif_body = elseif.body.into_iter()
                .map(convert_statement)
                .collect::<Result<Vec<_>>>()?;
            
            let nested_if = ast::EchoAst::If {
                condition: Box::new(elseif_condition),
                then_branch: elseif_body,
                else_branch: current_else,
            };
            
            current_else = Some(vec![nested_if]);
        }
        
        current_else
    } else {
        if let Some(else_clause) = if_stmt.else_clause {
            Some(else_clause.body.into_iter()
                .map(convert_statement)
                .collect::<Result<Vec<_>>>()?)
        } else {
            None
        }
    };
    
    Ok(ast::EchoAst::If {
        condition: Box::new(condition),
        then_branch,
        else_branch,
    })
}

fn convert_while_statement(while_stmt: g::WhileStatement) -> Result<ast::EchoAst> {
    let condition = convert_expression(*while_stmt.condition)?;
    let body = while_stmt.body.into_iter()
        .map(convert_statement)
        .collect::<Result<Vec<_>>>()?;
    
    let label = while_stmt.label.map(|id| "placeholder".to_string());
    
    Ok(ast::EchoAst::While {
        label,
        condition: Box::new(condition),
        body,
    })
}

fn convert_for_statement(for_stmt: g::ForStatement) -> Result<ast::EchoAst> {
    let variable = match *for_stmt.pattern {
        g::Pattern::Identifier(id) => "placeholder".to_string(),
        _ => bail!("For loops only support simple identifiers in existing AST"),
    };
    
    let collection = convert_expression(*for_stmt.iterable)?;
    let body = for_stmt.body.into_iter()
        .map(convert_statement)
        .collect::<Result<Vec<_>>>()?;
    
    Ok(ast::EchoAst::For {
        label: None,
        variable,
        collection: Box::new(collection),
        body,
    })
}

fn convert_return_statement(ret_stmt: g::ReturnStatement) -> Result<ast::EchoAst> {
    let value = if let Some(expr) = ret_stmt.value {
        Some(Box::new(convert_expression(*expr)?))
    } else {
        None
    };
    Ok(ast::EchoAst::Return { value })
}

fn convert_break_statement(break_stmt: g::BreakStatement) -> Result<ast::EchoAst> {
    Ok(ast::EchoAst::Break { 
        label: break_stmt.label.map(|id| "placeholder".to_string()) 
    })
}

fn convert_continue_statement(cont_stmt: g::ContinueStatement) -> Result<ast::EchoAst> {
    Ok(ast::EchoAst::Continue { 
        label: cont_stmt.label.map(|id| "placeholder".to_string()) 
    })
}

fn convert_list_literal(list: g::ListLiteral) -> Result<ast::EchoAst> {
    let elements = list.elements.into_iter()
        .map(|elem| match elem {
            g::ListElement::Expression(expr) => convert_expression(*expr),
            g::ListElement::Scatter { expression, .. } => {
                // For now, just convert the expression
                // TODO: Handle scatter properly
                convert_expression(*expression)
            }
        })
        .collect::<Result<Vec<_>>>()?;
    
    Ok(ast::EchoAst::List { elements })
}

// Binary operation converters
fn convert_binary_or(bin: g::BinaryExpression) -> Result<ast::EchoAst> {
    let left = convert_expression(*bin.left)?;
    let right = convert_expression(*bin.right)?;
    Ok(ast::EchoAst::Or {
        left: Box::new(left),
        right: Box::new(right),
    })
}

fn convert_binary_and(bin: g::BinaryExpression) -> Result<ast::EchoAst> {
    let left = convert_expression(*bin.left)?;
    let right = convert_expression(*bin.right)?;
    Ok(ast::EchoAst::And {
        left: Box::new(left),
        right: Box::new(right),
    })
}

fn convert_binary_equal(bin: g::BinaryExpression) -> Result<ast::EchoAst> {
    let left = convert_expression(*bin.left)?;
    let right = convert_expression(*bin.right)?;
    Ok(ast::EchoAst::Equal {
        left: Box::new(left),
        right: Box::new(right),
    })
}

fn convert_binary_not_equal(bin: g::BinaryExpression) -> Result<ast::EchoAst> {
    let left = convert_expression(*bin.left)?;
    let right = convert_expression(*bin.right)?;
    Ok(ast::EchoAst::NotEqual {
        left: Box::new(left),
        right: Box::new(right),
    })
}

fn convert_binary_less(bin: g::BinaryExpression) -> Result<ast::EchoAst> {
    let left = convert_expression(*bin.left)?;
    let right = convert_expression(*bin.right)?;
    Ok(ast::EchoAst::LessThan {
        left: Box::new(left),
        right: Box::new(right),
    })
}

fn convert_binary_less_equal(bin: g::BinaryExpression) -> Result<ast::EchoAst> {
    let left = convert_expression(*bin.left)?;
    let right = convert_expression(*bin.right)?;
    Ok(ast::EchoAst::LessEqual {
        left: Box::new(left),
        right: Box::new(right),
    })
}

fn convert_binary_greater(bin: g::BinaryExpression) -> Result<ast::EchoAst> {
    let left = convert_expression(*bin.left)?;
    let right = convert_expression(*bin.right)?;
    Ok(ast::EchoAst::GreaterThan {
        left: Box::new(left),
        right: Box::new(right),
    })
}

fn convert_binary_greater_equal(bin: g::BinaryExpression) -> Result<ast::EchoAst> {
    let left = convert_expression(*bin.left)?;
    let right = convert_expression(*bin.right)?;
    Ok(ast::EchoAst::GreaterEqual {
        left: Box::new(left),
        right: Box::new(right),
    })
}

fn convert_binary_add(bin: g::BinaryExpression) -> Result<ast::EchoAst> {
    let left = convert_expression(*bin.left)?;
    let right = convert_expression(*bin.right)?;
    Ok(ast::EchoAst::Add {
        left: Box::new(left),
        right: Box::new(right),
    })
}

fn convert_binary_subtract(bin: g::BinaryExpression) -> Result<ast::EchoAst> {
    let left = convert_expression(*bin.left)?;
    let right = convert_expression(*bin.right)?;
    Ok(ast::EchoAst::Subtract {
        left: Box::new(left),
        right: Box::new(right),
    })
}

fn convert_binary_multiply(bin: g::BinaryExpression) -> Result<ast::EchoAst> {
    let left = convert_expression(*bin.left)?;
    let right = convert_expression(*bin.right)?;
    Ok(ast::EchoAst::Multiply {
        left: Box::new(left),
        right: Box::new(right),
    })
}

fn convert_binary_divide(bin: g::BinaryExpression) -> Result<ast::EchoAst> {
    let left = convert_expression(*bin.left)?;
    let right = convert_expression(*bin.right)?;
    Ok(ast::EchoAst::Divide {
        left: Box::new(left),
        right: Box::new(right),
    })
}

fn convert_binary_modulo(bin: g::BinaryExpression) -> Result<ast::EchoAst> {
    let left = convert_expression(*bin.left)?;
    let right = convert_expression(*bin.right)?;
    Ok(ast::EchoAst::Modulo {
        left: Box::new(left),
        right: Box::new(right),
    })
}

fn convert_binary_power(bin: g::BinaryExpression) -> Result<ast::EchoAst> {
    let left = convert_expression(*bin.left)?;
    let right = convert_expression(*bin.right)?;
    Ok(ast::EchoAst::Power {
        left: Box::new(left),
        right: Box::new(right),
    })
}

fn convert_unary_not(unary: g::UnaryExpression) -> Result<ast::EchoAst> {
    let operand = convert_expression(*unary.operand)?;
    Ok(ast::EchoAst::Not {
        operand: Box::new(operand),
    })
}

fn convert_unary_negate(unary: g::UnaryExpression) -> Result<ast::EchoAst> {
    let operand = convert_expression(*unary.operand)?;
    Ok(ast::EchoAst::UnaryMinus {
        operand: Box::new(operand),
    })
}

fn convert_assignment_expression(assign: g::AssignmentExpression) -> Result<ast::EchoAst> {
    let target = convert_expression(*assign.target)?;
    let value = convert_expression(*assign.value)?;
    
    // Convert target to LValue
    let lvalue = expression_to_lvalue(target)?;
    
    Ok(ast::EchoAst::Assignment {
        target: lvalue,
        value: Box::new(value),
    })
}

fn expression_to_lvalue(expr: ast::EchoAst) -> Result<LValue> {
    match expr {
        ast::EchoAst::Identifier(name) => Ok(LValue::Binding {
            binding_type: BindingType::None,
            pattern: BindingPattern::Identifier(name),
        }),
        ast::EchoAst::PropertyAccess { object, property } => Ok(LValue::PropertyAccess {
            object,
            property,
        }),
        ast::EchoAst::IndexAccess { object, index } => Ok(LValue::IndexAccess {
            object,
            index,
        }),
        _ => bail!("Invalid assignment target"),
    }
}

fn convert_property_access(access: g::PropertyAccess) -> Result<ast::EchoAst> {
    let object = convert_expression(*access.object)?;
    Ok(ast::EchoAst::PropertyAccess {
        object: Box::new(object),
        property: "placeholder".to_string(),
    })
}

fn convert_method_call(call: g::MethodCall) -> Result<ast::EchoAst> {
    let object = convert_expression(*call.object)?;
    let args = call.arguments.into_iter()
        .map(convert_expression)
        .collect::<Result<Vec<_>>>()?;
    
    Ok(ast::EchoAst::MethodCall {
        object: Box::new(object),
        method: "placeholder".to_string(),
        args,
    })
}

fn convert_index_access(access: g::IndexAccess) -> Result<ast::EchoAst> {
    let object = convert_expression(*access.object)?;
    let index = convert_expression(*access.index)?;
    
    Ok(ast::EchoAst::IndexAccess {
        object: Box::new(object),
        index: Box::new(index),
    })
}

fn convert_call_expression(call: g::CallExpression) -> Result<ast::EchoAst> {
    let func = convert_expression(*call.function)?;
    let args = call.arguments.into_iter()
        .map(|arg| match arg {
            g::CallArgument::Expression(expr) => convert_expression(*expr),
            g::CallArgument::Scatter { expression, .. } => {
                // TODO: Handle scatter properly
                convert_expression(*expression)
            }
        })
        .collect::<Result<Vec<_>>>()?;
    
    Ok(ast::EchoAst::Call {
        func: Box::new(func),
        args,
    })
}

fn convert_lambda_expression(lambda: g::LambdaExpression) -> Result<ast::EchoAst> {
    let params = convert_pattern_to_lambda_params(*lambda.parameters)?;
    let body = match lambda.body {
        g::LambdaBody::Expression(expr) => Box::new(convert_expression(*expr)?),
        g::LambdaBody::Block { statements, .. } => {
            let converted_stmts = statements.into_iter()
                .map(convert_statement)
                .collect::<Result<Vec<_>>>()?;
            Box::new(ast::EchoAst::Block(converted_stmts))
        }
    };
    
    Ok(ast::EchoAst::Lambda {
        params,
        body,
    })
}

fn convert_pattern_to_lambda_params(pattern: g::Pattern) -> Result<Vec<ast::LambdaParam>> {
    match pattern {
        g::Pattern::Identifier(id) => {
            Ok(vec![ast::LambdaParam::Simple("placeholder".to_string())])
        }
        g::Pattern::List { elements, .. } => {
            elements.into_iter()
                .map(|elem| match elem {
                    g::PatternElement::Simple(id) => Ok(ast::LambdaParam::Simple("placeholder".to_string())),
                    g::PatternElement::Optional { name, default, .. } => {
                        let default_expr = convert_expression(*default)?;
                        Ok(ast::LambdaParam::Optional {
                            name: "placeholder".to_string(),
                            default: Box::new(default_expr),
                        })
                    }
                    g::PatternElement::Rest { name, .. } => {
                        Ok(ast::LambdaParam::Rest("placeholder".to_string()))
                    }
                })
                .collect()
        }
        g::Pattern::Rest { name, .. } => {
            Ok(vec![ast::LambdaParam::Rest("placeholder".to_string())])
        }
        g::Pattern::Ignore => {
            Ok(vec![ast::LambdaParam::Simple("_".to_string())])
        }
    }
}

fn convert_function_expression(func: g::FunctionExpression) -> Result<ast::EchoAst> {
    // Functions in the improved grammar map to lambdas in the existing AST
    let params = convert_pattern_to_lambda_params(*func.parameters)?;
    let body = if func.body.is_empty() {
        Box::new(ast::EchoAst::Null)
    } else {
        let converted_stmts = func.body.into_iter()
            .map(convert_statement)
            .collect::<Result<Vec<_>>>()?;
        Box::new(ast::EchoAst::Block(converted_stmts))
    };
    
    Ok(ast::EchoAst::Lambda {
        params,
        body,
    })
}

fn convert_object_definition(obj: g::ObjectDefinition) -> Result<ast::EchoAst> {
    let parent = obj.parent.map(|p| match *p.parent {
        g::Expression::Identifier(id) => Ok("placeholder".to_string()),
        _ => bail!("Object parent must be an identifier"),
    }).transpose()?;
    
    let members = obj.members.into_iter()
        .filter_map(|member| convert_object_member(member).ok())
        .collect();
    
    Ok(ast::EchoAst::ObjectDef {
        name: "placeholder".to_string(),
        parent,
        members,
    })
}

fn convert_object_member(member: g::ObjectMember) -> Result<ObjectMember> {
    match member {
        g::ObjectMember::Property(prop) => {
            let value = convert_expression(*prop.value)?;
            Ok(ObjectMember::Property {
                name: "placeholder".to_string(),
                value,
                permissions: None,
            })
        }
        g::ObjectMember::Verb(verb) => {
            let args = convert_pattern_to_parameters(*verb.parameters)?;
            let body = verb.body.into_iter()
                .map(convert_statement)
                .collect::<Result<Vec<_>>>()?;
            
            Ok(ObjectMember::Verb {
                name: "placeholder".to_string(),
                args,
                body,
                permissions: None,
            })
        }
        g::ObjectMember::Event(event) => {
            let params = convert_pattern_to_parameters(*event.parameters)?;
            let body = event.body.into_iter()
                .map(convert_statement)
                .collect::<Result<Vec<_>>>()?;
            
            Ok(ObjectMember::Event {
                name: "placeholder".to_string(),
                params,
                body,
            })
        }
        g::ObjectMember::Query(query) => {
            let params = if let Some(p) = query.parameters {
                // Extract variable names from the pattern
                match *p {
                    g::Pattern::Identifier(_) => vec!["placeholder".to_string()],
                    g::Pattern::List { elements, .. } => {
                        elements.into_iter()
                            .filter_map(|elem| match elem {
                                g::PatternElement::Simple(_) => Some("placeholder".to_string()),
                                _ => None,
                            })
                            .collect()
                    }
                    _ => vec![],
                }
            } else {
                vec![]
            };
            
            let clauses = query.clauses.into_iter()
                .map(|clause| Ok(ast::QueryClause {
                    predicate: "placeholder".to_string(),
                    args: vec![], // TODO: Parse query clause arguments
                }))
                .collect::<Result<Vec<_>>>()?;
            
            Ok(ObjectMember::Query {
                name: "placeholder".to_string(),
                params,
                clauses,
            })
        }
    }
}

fn convert_pattern_to_parameters(pattern: g::Pattern) -> Result<Vec<Parameter>> {
    match pattern {
        g::Pattern::Identifier(id) => {
            Ok(vec![Parameter {
                name: "placeholder".to_string(),
                type_annotation: None,
                default_value: None,
            }])
        }
        g::Pattern::List { elements, .. } => {
            elements.into_iter()
                .map(|elem| match elem {
                    g::PatternElement::Simple(id) => Ok(Parameter {
                        name: "placeholder".to_string(),
                        type_annotation: None,
                        default_value: None,
                    }),
                    g::PatternElement::Optional { name, default, .. } => {
                        let default_expr = convert_expression(*default)?;
                        Ok(Parameter {
                            name: "placeholder".to_string(),
                            type_annotation: None,
                            default_value: Some(default_expr),
                        })
                    }
                    g::PatternElement::Rest { name, .. } => Ok(Parameter {
                        name: "@placeholder".to_string(),
                        type_annotation: None,
                        default_value: None,
                    }),
                })
                .collect()
        }
        g::Pattern::Rest { .. } => {
            bail!("Rest patterns not supported as top-level parameter pattern")
        }
        g::Pattern::Ignore => {
            Ok(vec![])
        }
    }
}