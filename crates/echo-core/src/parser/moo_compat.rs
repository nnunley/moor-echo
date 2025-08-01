use anyhow::{anyhow, Result};
use std::collections::HashMap;

use crate::ast::{EchoAst, ObjectMember, Parameter, VerbPermissions, LValue, BindingType, BindingPattern, DestructuringTarget};
use crate::parser::Parser;

/// MOO compatibility parser using tree-sitter-moo
pub struct MooCompatParser {
    parser: tree_sitter::Parser,
}

impl MooCompatParser {
    pub fn new() -> Result<Self> {
        let mut parser = tree_sitter::Parser::new();
        let language = tree_sitter_moo::language();
        parser
            .set_language(&language)
            .map_err(|e| anyhow!("Failed to set MOO language: {}", e))?;
        
        Ok(Self { parser })
    }

    fn convert_node(&self, node: tree_sitter::Node, source: &str) -> Result<EchoAst> {
        match node.kind() {
            "program" => {
                let mut statements = Vec::new();
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if child.kind() != "comment" {
                        let stmt = self.convert_node(child, source)?;
                        // Filter out placeholder Null values (e.g., from standalone "define")
                        if !matches!(stmt, EchoAst::Null) {
                            statements.push(stmt);
                        }
                    }
                }
                Ok(EchoAst::Program(statements))
            }
            
            "object_definition" => {
                self.convert_object_definition(node, source)
            }
            
            "verb_definition" => {
                // Verb definitions are handled within object definitions
                Err(anyhow!("Standalone verb definitions not supported"))
            }
            
            "property_definition" => {
                self.convert_property_definition(node, source)
            }
            
            "assignment" => {
                self.convert_assignment(node, source)
            }
            
            "if_statement" => {
                self.convert_if_statement(node, source)
            }
            
            "while_statement" => {
                self.convert_while_statement(node, source)
            }
            
            "for_statement" => {
                self.convert_for_statement(node, source)
            }
            
            "return_statement" => {
                self.convert_return_statement(node, source)
            }
            
            "expression_statement" => {
                let expr_node = node.child(0).ok_or_else(|| anyhow!("Missing expression"))?;
                
                // Check if this is a define statement masquerading as an expression
                if expr_node.kind() == "expression" && expr_node.child_count() > 0 {
                    let first_child = expr_node.child(0).unwrap();
                    if first_child.kind() == "identifier" && self.get_node_text(first_child, source) == "define" {
                        // This is a define statement - look for the assignment that follows
                        if let Some(parent) = node.parent() {
                            let mut found_assignment = false;
                            let mut cursor = parent.walk();
                            let mut next_node = None;
                            
                            // Find the current node and then get the next one
                            for child in parent.children(&mut cursor) {
                                if found_assignment {
                                    next_node = Some(child);
                                    break;
                                }
                                if child.id() == node.id() {
                                    found_assignment = true;
                                }
                            }
                            
                            if let Some(next) = next_node {
                                if let Some(assignment) = next.child(0).and_then(|n| n.child(0)) {
                                    if assignment.kind() == "assignment_operation" {
                                        let name_node = assignment.child(0);
                                        let value_node = assignment.child(2);
                                        
                                        if let (Some(name), Some(value)) = (name_node, value_node) {
                                            return Ok(EchoAst::Define {
                                                name: self.get_node_text(name, source).to_string(),
                                                value: Box::new(self.convert_expression(value, source)?),
                                            });
                                        }
                                    }
                                }
                            }
                        }
                        
                        // If we didn't find the assignment, this is just a standalone "define"
                        // which we should skip (it will be paired with the next statement)
                        return Ok(EchoAst::Null);
                    }
                }
                
                self.convert_expression(expr_node, source)
            }
            
            "define_statement" => {
                self.convert_define_statement(node, source)
            }
            
            _ => self.convert_expression(node, source),
        }
    }

    fn convert_object_definition(&self, node: tree_sitter::Node, source: &str) -> Result<EchoAst> {
        let mut name = None;
        let mut parent = None;
        let mut members = Vec::new();
        
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "identifier" => {
                    if name.is_none() {
                        name = Some(self.get_node_text(child, source).to_string());
                    }
                }
                "parent_spec" => {
                    if let Some(parent_id) = child.child(1) {
                        // Extract parent name from the expression
                        match self.convert_expression(parent_id, source)? {
                            EchoAst::Identifier(name) => parent = Some(name),
                            _ => parent = Some("$root".to_string()), // Default to root if not an identifier
                        }
                    }
                }
                "object_body" => {
                    let mut body_cursor = child.walk();
                    for member in child.children(&mut body_cursor) {
                        match member.kind() {
                            "verb_definition" => {
                                if let Some((name, args, body, permissions)) = self.convert_verb_definition_parts(member, source)? {
                                    members.push(ObjectMember::Verb {
                                        name,
                                        args,
                                        body,
                                        permissions,
                                        required_capabilities: Vec::new(),
                                    });
                                }
                            }
                            "property_definition" => {
                                let prop = self.convert_property_definition(member, source)?;
                                if let EchoAst::Assignment { target, value } = prop {
                                    // Extract property name from the target
                                    let prop_name = match target {
                                        LValue::Binding { pattern, .. } => match pattern {
                                            BindingPattern::Identifier(name) => name,
                                            _ => "unnamed".to_string(),
                                        },
                                        _ => "unnamed".to_string(),
                                    };
                                    members.push(ObjectMember::Property {
                                        name: prop_name,
                                        value: *value,
                                        permissions: None,
                                        required_capabilities: Vec::new(),
                                    });
                                }
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }
        
        Ok(EchoAst::ObjectDef {
            name: name.unwrap_or_else(|| "anonymous".to_string()),
            parent,
            members,
        })
    }

    fn convert_verb_definition_parts(&self, node: tree_sitter::Node, source: &str) -> Result<Option<(String, Vec<Parameter>, Vec<EchoAst>, Option<VerbPermissions>)>> {
        let mut name = None;
        let mut args = Vec::new();
        let mut body = Vec::new();
        let mut permissions = None;
        
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "verb_name" => {
                    name = Some(self.get_node_text(child, source).to_string());
                }
                "verb_args" => {
                    args = self.convert_verb_args(child, source)?;
                }
                "verb_permissions" => {
                    permissions = Some(self.convert_verb_permissions(child, source)?);
                }
                "verb_body" => {
                    let mut body_cursor = child.walk();
                    for stmt in child.children(&mut body_cursor) {
                        if stmt.kind() != "comment" {
                            body.push(self.convert_node(stmt, source)?);
                        }
                    }
                }
                _ => {}
            }
        }
        
        if let Some(name) = name {
            Ok(Some((name, args, body, permissions)))
        } else {
            Ok(None)
        }
    }

    fn convert_property_definition(&self, node: tree_sitter::Node, source: &str) -> Result<EchoAst> {
        let mut name = None;
        let mut value = None;
        
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "property_name" => {
                    name = Some(self.get_node_text(child, source).to_string());
                }
                "property_value" => {
                    if let Some(expr) = child.child(0) {
                        value = Some(self.convert_expression(expr, source)?);
                    }
                }
                _ => {}
            }
        }
        
        let name = name.unwrap_or_else(|| "unnamed_property".to_string());
        let value = value.unwrap_or(EchoAst::Null);
        
        Ok(EchoAst::Assignment {
            target: LValue::Binding {
                binding_type: BindingType::Let,
                pattern: BindingPattern::Identifier(name),
            },
            value: Box::new(value),
        })
    }

    fn convert_expression(&self, node: tree_sitter::Node, source: &str) -> Result<EchoAst> {
        match node.kind() {
            "identifier" => {
                let text = self.get_node_text(node, source);
                Ok(EchoAst::Identifier(text.to_string()))
            }
            
            "object_id" => {
                let text = self.get_node_text(node, source);
                // Object IDs are in the form #123 or #-123
                if text.starts_with('#') {
                    let num_str = &text[1..];
                    if let Ok(num) = num_str.parse::<i64>() {
                        Ok(EchoAst::ObjectRef(num))
                    } else {
                        Err(anyhow!("Invalid object ID: {}", text))
                    }
                } else {
                    Err(anyhow!("Object ID must start with #: {}", text))
                }
            }
            
            "integer" => {
                let text = self.get_node_text(node, source);
                let value = text.parse::<i64>()
                    .map_err(|e| anyhow!("Failed to parse integer: {}", e))?;
                Ok(EchoAst::Number(value))
            }
            
            "float" => {
                let text = self.get_node_text(node, source);
                let value = text.parse::<f64>()
                    .map_err(|e| anyhow!("Failed to parse float: {}", e))?;
                Ok(EchoAst::Float(value))
            }
            
            "string" => {
                let text = self.get_node_text(node, source);
                // Remove quotes
                let content = text.trim_start_matches('"').trim_end_matches('"');
                Ok(EchoAst::String(content.to_string()))
            }
            
            "list" => {
                let mut elements = Vec::new();
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if child.kind() != "{" && child.kind() != "}" && child.kind() != "," {
                        elements.push(self.convert_expression(child, source)?);
                    }
                }
                Ok(EchoAst::List { elements })
            }
            
            "property_access" => {
                self.convert_property_access(node, source)
            }
            
            "method_call" => {
                self.convert_method_call(node, source)
            }
            
            "call" | "function_call" => {
                self.convert_function_call(node, source)
            }
            
            "binary_expression" => {
                self.convert_binary_expression(node, source)
            }
            
            "unary_expression" => {
                self.convert_unary_expression(node, source)
            }
            
            "assignment_expression" => {
                self.convert_assignment(node, source)
            }
            
            "flyweight" | "flyweight_expression" => {
                self.convert_flyweight(node, source)
            }
            
            "error_catch" | "error_catch_expression" => {
                self.convert_error_catch(node, source)
            }
            
            "map" | "map_literal" => {
                self.convert_map_literal(node, source)
            }
            
            "spread" | "spread_expression" => {
                let expr = node.child(1).ok_or_else(|| anyhow!("Missing expression in spread"))?;
                Ok(EchoAst::Spread {
                    expr: Box::new(self.convert_expression(expr, source)?),
                })
            }
            
            "destructuring_assignment" => {
                self.convert_destructuring(node, source)
            }
            
            "symbol" | "symbol_literal" => {
                let text = self.get_node_text(node, source);
                // Remove the leading quote
                let symbol_name = text.trim_start_matches('\'');
                Ok(EchoAst::Identifier(format!(":{}", symbol_name))) // Prefix with : to indicate symbol
            }
            
            "statement" => {
                // For constants.moo - statement nodes contain expression statements
                if let Some(child) = node.child(0) {
                    self.convert_node(child, source)
                } else {
                    Err(anyhow!("Empty statement node"))
                }
            }
            
            "expression" => {
                // An expression node can contain various expression types
                if node.child_count() == 1 {
                    let child = node.child(0).unwrap();
                    
                    // Special handling for assignment operations
                    if child.kind() == "assignment_operation" {
                        let left = child.child(0).ok_or_else(|| anyhow!("Missing assignment target"))?;
                        let right = child.child(2).ok_or_else(|| anyhow!("Missing assignment value"))?;
                        
                        // Create an Assignment node
                        let target = LValue::Binding {
                            binding_type: BindingType::None,
                            pattern: BindingPattern::Identifier(self.get_node_text(left, source).to_string()),
                        };
                        
                        return Ok(EchoAst::Assignment {
                            target,
                            value: Box::new(self.convert_expression(right, source)?),
                        });
                    }
                    
                    self.convert_expression(child, source)
                } else {
                    Err(anyhow!("Unexpected expression structure"))
                }
            }
            
            "ERROR" => {
                // Tree-sitter couldn't parse this properly - try to extract what we can
                let text = self.get_node_text(node, source);
                
                // Special handling for common patterns
                if text.starts_with("define ") {
                    // Handle define statements that weren't parsed correctly
                    let parts: Vec<&str> = text.split_whitespace().collect();
                    if parts.len() >= 4 && parts[2] == "=" {
                        let name = parts[1].to_string();
                        let joined_value = parts[3..].join(" ");
                        let value_text = joined_value.trim_end_matches(';');
                        
                        // Try to parse the value
                        let value = if value_text.starts_with('#') {
                            // Object reference
                            if let Ok(num) = value_text[1..].parse::<i64>() {
                                EchoAst::ObjectRef(num)
                            } else {
                                EchoAst::String(value_text.to_string())
                            }
                        } else {
                            EchoAst::String(value_text.to_string())
                        };
                        
                        return Ok(EchoAst::Define {
                            name,
                            value: Box::new(value),
                        });
                    }
                }
                
                // Try to find valid child nodes within the ERROR node
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if child.kind() != "ERROR" {
                        // Try to parse any non-ERROR child
                        if let Ok(ast) = self.convert_node(child, source) {
                            return Ok(ast);
                        }
                    }
                }
                
                Err(anyhow!("Cannot parse ERROR node: {}", text.chars().take(100).collect::<String>()))
            }
            
            _ => {
                Err(anyhow!("Unsupported expression type: {}", node.kind()))
            }
        }
    }

    fn convert_property_access(&self, node: tree_sitter::Node, source: &str) -> Result<EchoAst> {
        let object = node.child(0).ok_or_else(|| anyhow!("Missing object in property access"))?;
        let property = node.child(2).ok_or_else(|| anyhow!("Missing property in property access"))?;
        
        Ok(EchoAst::PropertyAccess {
            object: Box::new(self.convert_expression(object, source)?),
            property: self.get_node_text(property, source).to_string(),
        })
    }

    fn convert_method_call(&self, node: tree_sitter::Node, source: &str) -> Result<EchoAst> {
        let object = node.child(0).ok_or_else(|| anyhow!("Missing object in method call"))?;
        let method = node.child(2).ok_or_else(|| anyhow!("Missing method in method call"))?;
        let mut args = Vec::new();
        
        // Find the argument list
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "argument_list" {
                let mut arg_cursor = child.walk();
                for arg in child.children(&mut arg_cursor) {
                    if arg.kind() != "(" && arg.kind() != ")" && arg.kind() != "," {
                        args.push(self.convert_expression(arg, source)?);
                    }
                }
            }
        }
        
        Ok(EchoAst::MethodCall {
            object: Box::new(self.convert_expression(object, source)?),
            method: self.get_node_text(method, source).to_string(),
            args,
        })
    }
    
    fn convert_function_call(&self, node: tree_sitter::Node, source: &str) -> Result<EchoAst> {
        let function = node.child(0).ok_or_else(|| anyhow!("Missing function name in call"))?;
        let mut args = Vec::new();
        
        // Find the argument list - might be child(1) or we need to search
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "argument_list" || child.kind() == "arguments" {
                let mut arg_cursor = child.walk();
                for arg in child.children(&mut arg_cursor) {
                    if arg.kind() != "(" && arg.kind() != ")" && arg.kind() != "," {
                        args.push(self.convert_expression(arg, source)?);
                    }
                }
                break;
            }
        }
        
        // If no argument list found, check if child(1) is the arguments directly
        if args.is_empty() {
            if let Some(args_node) = node.child(1) {
                if args_node.kind() == "argument_list" || args_node.kind() == "arguments" {
                    let mut arg_cursor = args_node.walk();
                    for arg in args_node.children(&mut arg_cursor) {
                        if arg.kind() != "(" && arg.kind() != ")" && arg.kind() != "," {
                            args.push(self.convert_expression(arg, source)?);
                        }
                    }
                }
            }
        }
        
        Ok(EchoAst::Call {
            func: Box::new(self.convert_expression(function, source)?),
            args,
        })
    }

    fn convert_binary_expression(&self, node: tree_sitter::Node, source: &str) -> Result<EchoAst> {
        let left = node.child(0).ok_or_else(|| anyhow!("Missing left operand"))?;
        let op = node.child(1).ok_or_else(|| anyhow!("Missing operator"))?;
        let right = node.child(2).ok_or_else(|| anyhow!("Missing right operand"))?;
        
        let op_text = self.get_node_text(op, source);
        let left_expr = Box::new(self.convert_expression(left, source)?);
        let right_expr = Box::new(self.convert_expression(right, source)?);
        
        match op_text {
            "+" => Ok(EchoAst::Add { left: left_expr, right: right_expr }),
            "-" => Ok(EchoAst::Subtract { left: left_expr, right: right_expr }),
            "*" => Ok(EchoAst::Multiply { left: left_expr, right: right_expr }),
            "/" => Ok(EchoAst::Divide { left: left_expr, right: right_expr }),
            "%" => Ok(EchoAst::Modulo { left: left_expr, right: right_expr }),
            "==" => Ok(EchoAst::Equal { left: left_expr, right: right_expr }),
            "!=" => Ok(EchoAst::NotEqual { left: left_expr, right: right_expr }),
            "<" => Ok(EchoAst::LessThan { left: left_expr, right: right_expr }),
            ">" => Ok(EchoAst::GreaterThan { left: left_expr, right: right_expr }),
            "<=" => Ok(EchoAst::LessEqual { left: left_expr, right: right_expr }),
            ">=" => Ok(EchoAst::GreaterEqual { left: left_expr, right: right_expr }),
            "&&" => Ok(EchoAst::And { left: left_expr, right: right_expr }),
            "||" => Ok(EchoAst::Or { left: left_expr, right: right_expr }),
            _ => Err(anyhow!("Unknown binary operator: {}", op_text)),
        }
    }

    fn convert_unary_expression(&self, node: tree_sitter::Node, source: &str) -> Result<EchoAst> {
        let op = node.child(0).ok_or_else(|| anyhow!("Missing operator"))?;
        let operand = node.child(1).ok_or_else(|| anyhow!("Missing operand"))?;
        
        let op_text = self.get_node_text(op, source);
        let operand_expr = Box::new(self.convert_expression(operand, source)?);
        
        match op_text {
            "!" => Ok(EchoAst::Not { operand: operand_expr }),
            "-" => Ok(EchoAst::Subtract { 
                left: Box::new(EchoAst::Number(0)),
                right: operand_expr,
            }),  // Simulate negation as 0 - expr
            _ => Err(anyhow!("Unknown unary operator: {}", op_text)),
        }
    }

    fn convert_assignment(&self, node: tree_sitter::Node, source: &str) -> Result<EchoAst> {
        let target = node.child(0).ok_or_else(|| anyhow!("Missing assignment target"))?;
        let value = node.child(2).ok_or_else(|| anyhow!("Missing assignment value"))?;
        
        // Check if this is a destructuring assignment
        if target.kind() == "list" || target.kind() == "destructuring_pattern" {
            let mut targets = Vec::new();
            let mut cursor = target.walk();
            
            for child in target.children(&mut cursor) {
                if child.kind() == "identifier" {
                    let name = self.get_node_text(child, source).to_string();
                    targets.push(DestructuringTarget::Simple(name));
                } else if child.kind() == "optional_parameter" {
                    // Handle ?param = default syntax
                    if let Some(name_node) = child.child_by_field_name("name") {
                        let name = self.get_node_text(name_node, source).to_string();
                        let default = if let Some(default_node) = child.child_by_field_name("default") {
                            Box::new(self.convert_expression(default_node, source)?)
                        } else {
                            Box::new(EchoAst::Null)
                        };
                        targets.push(DestructuringTarget::Optional { name, default });
                    }
                }
            }
            
            return Ok(EchoAst::DestructuringAssignment {
                targets,
                value: Box::new(self.convert_expression(value, source)?),
            });
        }
        
        let target_text = self.get_node_text(target, source);
        Ok(EchoAst::Assignment {
            target: LValue::Binding {
                binding_type: BindingType::Let,
                pattern: BindingPattern::Identifier(target_text.to_string()),
            },
            value: Box::new(self.convert_expression(value, source)?),
        })
    }

    fn convert_if_statement(&self, node: tree_sitter::Node, source: &str) -> Result<EchoAst> {
        let condition = node.child_by_field_name("condition")
            .ok_or_else(|| anyhow!("Missing if condition"))?;
        let then_branch = node.child_by_field_name("then")
            .ok_or_else(|| anyhow!("Missing then branch"))?;
        let else_branch = node.child_by_field_name("else");
        
        let then_body = match self.convert_block_or_statement(then_branch, source)? {
            EchoAst::Block(body) => body,
            stmt => vec![stmt],
        };
        let else_body = else_branch.map(|n| match self.convert_block_or_statement(n, source) {
            Ok(EchoAst::Block(body)) => Ok(body),
            Ok(stmt) => Ok(vec![stmt]),
            Err(e) => Err(e),
        }).transpose()?;
        
        Ok(EchoAst::If {
            condition: Box::new(self.convert_expression(condition, source)?),
            then_branch: then_body,
            else_branch: else_body,
        })
    }

    fn convert_while_statement(&self, node: tree_sitter::Node, source: &str) -> Result<EchoAst> {
        let condition = node.child_by_field_name("condition")
            .ok_or_else(|| anyhow!("Missing while condition"))?;
        let body = node.child_by_field_name("body")
            .ok_or_else(|| anyhow!("Missing while body"))?;
        
        let body_ast = match self.convert_block_or_statement(body, source)? {
            EchoAst::Block(body) => body,
            stmt => vec![stmt],
        };
        
        Ok(EchoAst::While {
            label: None,
            condition: Box::new(self.convert_expression(condition, source)?),
            body: body_ast,
        })
    }

    fn convert_for_statement(&self, node: tree_sitter::Node, source: &str) -> Result<EchoAst> {
        let var = node.child_by_field_name("variable")
            .ok_or_else(|| anyhow!("Missing for variable"))?;
        let list = node.child_by_field_name("list")
            .ok_or_else(|| anyhow!("Missing for list"))?;
        let body = node.child_by_field_name("body")
            .ok_or_else(|| anyhow!("Missing for body"))?;
        
        let body_ast = match self.convert_block_or_statement(body, source)? {
            EchoAst::Block(body) => body,
            stmt => vec![stmt],
        };
        
        Ok(EchoAst::For {
            label: None,
            variable: self.get_node_text(var, source).to_string(),
            collection: Box::new(self.convert_expression(list, source)?),
            body: body_ast,
        })
    }

    fn convert_return_statement(&self, node: tree_sitter::Node, source: &str) -> Result<EchoAst> {
        let value = node.child(1); // Skip "return" keyword
        let return_value = match value {
            Some(n) => Some(Box::new(self.convert_expression(n, source)?)),
            None => None,
        };
        
        Ok(EchoAst::Return { value: return_value })
    }

    fn convert_block_or_statement(&self, node: tree_sitter::Node, source: &str) -> Result<EchoAst> {
        if node.kind() == "block" {
            let mut statements = Vec::new();
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() != "{" && child.kind() != "}" && child.kind() != "comment" {
                    statements.push(self.convert_node(child, source)?);
                }
            }
            Ok(EchoAst::Block(statements))
        } else {
            self.convert_node(node, source)
        }
    }

    fn convert_verb_args(&self, node: tree_sitter::Node, source: &str) -> Result<Vec<Parameter>> {
        let mut args = Vec::new();
        let mut cursor = node.walk();
        
        for child in node.children(&mut cursor) {
            if child.kind() == "identifier" {
                args.push(Parameter {
                    name: self.get_node_text(child, source).to_string(),
                    default_value: None,
                    type_annotation: None,
                });
            }
        }
        
        Ok(args)
    }

    fn convert_verb_permissions(&self, node: tree_sitter::Node, source: &str) -> Result<VerbPermissions> {
        let perms_text = self.get_node_text(node, source);
        
        // MOO permissions are typically "r", "w", "x" or combinations
        Ok(VerbPermissions {
            read: if perms_text.contains('r') { "read".to_string() } else { "".to_string() },
            write: if perms_text.contains('w') { "write".to_string() } else { "".to_string() },
            execute: if perms_text.contains('x') { "execute".to_string() } else { "".to_string() },
        })
    }

    fn get_node_text<'a>(&self, node: tree_sitter::Node, source: &'a str) -> &'a str {
        &source[node.byte_range()]
    }

    fn convert_define_statement(&self, node: tree_sitter::Node, source: &str) -> Result<EchoAst> {
        let mut name = None;
        let mut value = None;
        
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "identifier" => {
                    if name.is_none() {
                        name = Some(self.get_node_text(child, source).to_string());
                    }
                }
                "=" => {} // Skip operator
                _ => {
                    // This should be the value expression
                    if value.is_none() && child.kind() != "define" {
                        value = Some(self.convert_expression(child, source)?);
                    }
                }
            }
        }
        
        Ok(EchoAst::Define {
            name: name.unwrap_or_else(|| "unnamed".to_string()),
            value: Box::new(value.unwrap_or(EchoAst::Null)),
        })
    }
    
    fn convert_flyweight(&self, node: tree_sitter::Node, source: &str) -> Result<EchoAst> {
        let object = node.child(1).ok_or_else(|| anyhow!("Missing object in flyweight"))?;
        let properties_node = node.child(3).ok_or_else(|| anyhow!("Missing properties in flyweight"))?;
        
        let mut properties = Vec::new();
        let mut cursor = properties_node.walk();
        
        for child in properties_node.children(&mut cursor) {
            if child.kind() == "map_entry" || child.kind() == "property_entry" {
                if let Some((key, value)) = self.convert_map_entry(child, source)? {
                    properties.push((key, value));
                }
            }
        }
        
        Ok(EchoAst::Flyweight {
            object: Box::new(self.convert_expression(object, source)?),
            properties,
        })
    }
    
    fn convert_map_entry(&self, node: tree_sitter::Node, source: &str) -> Result<Option<(String, EchoAst)>> {
        let key_node = node.child(0);
        let value_node = node.child(2); // Skip the -> operator
        
        if let (Some(key), Some(value)) = (key_node, value_node) {
            let key_text = match self.convert_expression(key, source)? {
                EchoAst::Identifier(s) => s,
                EchoAst::String(s) => s,
                _ => self.get_node_text(key, source).to_string(),
            };
            
            let value_ast = self.convert_expression(value, source)?;
            Ok(Some((key_text, value_ast)))
        } else {
            Ok(None)
        }
    }
    
    fn convert_error_catch(&self, node: tree_sitter::Node, source: &str) -> Result<EchoAst> {
        // MOO error catch syntax: `expr ! E_TYPE, E_PROPNF => default'
        let expr = node.child(1).ok_or_else(|| anyhow!("Missing expression in error catch"))?;
        
        let mut error_patterns = Vec::new();
        let mut default_expr = None;
        let mut found_arrow = false;
        
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "error_code" | "identifier" => {
                    if !found_arrow {
                        error_patterns.push(self.get_node_text(child, source).to_string());
                    }
                }
                "=>" => {
                    found_arrow = true;
                }
                _ => {
                    if found_arrow && default_expr.is_none() && child.kind() != "`" && child.kind() != "'" {
                        default_expr = Some(self.convert_expression(child, source)?);
                    }
                }
            }
        }
        
        Ok(EchoAst::ErrorCatch {
            expr: Box::new(self.convert_expression(expr, source)?),
            error_patterns,
            default: Box::new(default_expr.unwrap_or(EchoAst::Null)),
        })
    }
    
    fn convert_destructuring(&self, node: tree_sitter::Node, source: &str) -> Result<EchoAst> {
        // {a, b, c} = expr
        let targets_node = node.child(0).ok_or_else(|| anyhow!("Missing targets in destructuring"))?;
        let value = node.child(2).ok_or_else(|| anyhow!("Missing value in destructuring"))?;
        
        let mut targets = Vec::new();
        let mut cursor = targets_node.walk();
        
        for child in targets_node.children(&mut cursor) {
            if child.kind() == "identifier" {
                let name = self.get_node_text(child, source).to_string();
                targets.push(DestructuringTarget::Simple(name));
            } else if child.kind() == "optional_parameter" {
                // Handle ?param = default syntax
                if let Some(name_node) = child.child_by_field_name("name") {
                    let name = self.get_node_text(name_node, source).to_string();
                    let default = if let Some(default_node) = child.child_by_field_name("default") {
                        Box::new(self.convert_expression(default_node, source)?)
                    } else {
                        Box::new(EchoAst::Null)
                    };
                    targets.push(DestructuringTarget::Optional { name, default });
                }
            }
        }
        
        Ok(EchoAst::DestructuringAssignment {
            targets,
            value: Box::new(self.convert_expression(value, source)?),
        })
    }
    
    fn convert_map_literal(&self, node: tree_sitter::Node, source: &str) -> Result<EchoAst> {
        let mut entries = Vec::new();
        let mut cursor = node.walk();
        
        for child in node.children(&mut cursor) {
            if child.kind() == "map_entry" || child.kind() == "property_entry" {
                if let Some((key, value)) = self.convert_map_entry(child, source)? {
                    entries.push((key, value));
                }
            }
        }
        
        Ok(EchoAst::MapLiteral { entries })
    }
}

impl Parser for MooCompatParser {
    fn parse(&mut self, source: &str) -> Result<EchoAst> {
        let tree = self.parser.parse(source, None)
            .ok_or_else(|| anyhow!("Failed to parse MOO source"))?;
        
        let root = tree.root_node();
        self.convert_node(root, source)
    }

    fn name(&self) -> &'static str {
        "moo-compat"
    }
}

/// Import MOO objects from source code into the Echo object store
pub fn import_moo_objects(
    source: &str,
    store: &crate::storage::object_store::ObjectStore,
) -> Result<Vec<crate::storage::object_store::ObjectId>> {
    let mut parser = MooCompatParser::new()?;
    let ast = parser.parse(source)?;
    
    let mut imported_ids = Vec::new();
    
    // Process the AST to find object definitions
    match ast {
        EchoAst::Program(statements) => {
            for stmt in statements {
                if let EchoAst::ObjectDef { name, parent, members } = stmt {
                    let obj_id = crate::storage::object_store::ObjectId::new();
                    
                    // Create the Echo object
                    let mut properties = HashMap::new();
                    let property_capabilities = HashMap::new();
                    let mut verbs = HashMap::new();
                    
                    // Process members
                    for member in members {
                        match member {
                            ObjectMember::Property { name, value, .. } => {
                                // Convert AST value to PropertyValue
                                let prop_value = ast_to_property_value(&value)?;
                                properties.insert(name, prop_value);
                            }
                            ObjectMember::Verb { name: verb_name, args, body, permissions, .. } => {
                                let verb_def = crate::storage::object_store::VerbDefinition {
                                    name: verb_name.clone(),
                                    signature: crate::storage::object_store::VerbSignature {
                                        dobj: "any".to_string(),
                                        prep: "none".to_string(),
                                        iobj: "any".to_string(),
                                    },
                                    code: format!("{:?}", body), // Store AST as string for now
                                    ast: body,
                                    params: args,
                                    permissions: match permissions {
                                        Some(p) => crate::storage::object_store::VerbPermissions {
                                            read: !p.read.is_empty(),
                                            write: !p.write.is_empty(),
                                            execute: !p.execute.is_empty(),
                                        },
                                        None => crate::storage::object_store::VerbPermissions {
                                            read: true,
                                            write: false,
                                            execute: true,
                                        },
                                    },
                                    required_capabilities: Vec::new(),
                                };
                                verbs.insert(verb_name, verb_def);
                            }
                            _ => {} // Other member types not yet supported
                        }
                    }
                    
                    let parent_id = if let Some(parent_name) = parent {
                        // Try to resolve parent reference
                        store.find_by_name(&parent_name)?.or(Some(crate::storage::object_store::ObjectId::root()))
                    } else {
                        Some(crate::storage::object_store::ObjectId::root())
                    };
                    
                    let echo_obj = crate::storage::object_store::EchoObject {
                        id: obj_id,
                        parent: parent_id,
                        name,
                        properties,
                        property_capabilities,
                        verbs,
                        queries: HashMap::new(),
                        meta: crate::evaluator::meta_object::MetaObject::new(obj_id),
                    };
                    
                    store.store(echo_obj)?;
                    imported_ids.push(obj_id);
                }
            }
        }
        _ => return Err(anyhow!("Expected program containing object definitions")),
    }
    
    Ok(imported_ids)
}

fn ast_to_property_value(ast: &EchoAst) -> Result<crate::storage::object_store::PropertyValue> {
    use crate::storage::object_store::PropertyValue;
    
    match ast {
        EchoAst::Null => Ok(PropertyValue::Null),
        EchoAst::Boolean(b) => Ok(PropertyValue::Boolean(*b)),
        EchoAst::Number(i) => Ok(PropertyValue::Integer(*i)),
        EchoAst::Float(f) => Ok(PropertyValue::Float(*f)),
        EchoAst::String(s) => Ok(PropertyValue::String(s.clone())),
        EchoAst::List { elements } => {
            let values = elements.iter()
                .map(ast_to_property_value)
                .collect::<Result<Vec<_>>>()?;
            Ok(PropertyValue::List(values))
        }
        EchoAst::MapLiteral { entries } => {
            let mut map = HashMap::new();
            for (k, v) in entries {
                let value = ast_to_property_value(v)?;
                map.insert(k.clone(), value);
            }
            Ok(PropertyValue::Map(map))
        }
        _ => Err(anyhow!("Cannot convert AST node to property value: {:?}", ast)),
    }
}