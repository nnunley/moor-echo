use anyhow::{anyhow, Result};
use std::collections::HashMap;

use crate::ast::{EchoAst, ObjectMember, Parameter, VerbPermissions, LValue, BindingType, BindingPattern};
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
                        statements.push(self.convert_node(child, source)?);
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
                self.convert_expression(expr_node, source)
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
            
            "binary_expression" => {
                self.convert_binary_expression(node, source)
            }
            
            "unary_expression" => {
                self.convert_unary_expression(node, source)
            }
            
            "assignment_expression" => {
                self.convert_assignment(node, source)
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
        _ => Err(anyhow!("Cannot convert AST node to property value: {:?}", ast)),
    }
}