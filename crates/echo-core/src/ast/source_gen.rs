// Source code generation from AST
// This module provides functionality to reconstruct source code from the AST,
// similar to how LambdaMOO generates source from bytecode

use super::*;

/// Trait for types that can generate their source code representation
pub trait ToSource {
    fn to_source(&self) -> String;
}

// Helper function to format ObjectMember to reduce nesting
fn format_object_member(member: &ObjectMember) -> String {
    match member {
        ObjectMember::Property {
            name,
            value,
            permissions,
            required_capabilities,
        } => {
            let mut result = format!("  property {} = {}", name, value.to_source());
            if !required_capabilities.is_empty() {
                result.push_str(" requires ");
                result.push_str(&required_capabilities.join(", "));
            }
            if let Some(perms) = permissions {
                result.push_str(&format!(" {{{}+{}}}", perms.read, perms.write));
            }
            result.push('\n');
            result
        }
        ObjectMember::Verb {
            name,
            args,
            body,
            permissions,
            required_capabilities,
        } => {
            let mut result = format!("  verb {name}(");
            let args_str = args
                .iter()
                .map(|p| p.to_source())
                .collect::<Vec<_>>()
                .join(", ");
            result.push_str(&args_str);
            result.push_str(") ");
            if !required_capabilities.is_empty() {
                result.push_str("requires ");
                result.push_str(&required_capabilities.join(", "));
                result.push(' ');
            }
            if let Some(perms) = permissions {
                result.push_str(&format!(
                    "{{{}+{}+{}}}",
                    perms.read, perms.write, perms.execute
                ));
            }
            result.push('\n');
            for stmt in body {
                result.push_str(&format!("    {}\n", stmt.to_source()));
            }
            result.push_str("  endverb\n");
            result
        }
        ObjectMember::Method {
            name,
            args,
            body,
            return_type,
        } => {
            let mut result = format!("  method {name}(");
            let args_str = args
                .iter()
                .map(|p| p.to_source())
                .collect::<Vec<_>>()
                .join(", ");
            result.push_str(&args_str);
            result.push(')');
            if let Some(ret_type) = return_type {
                result.push_str(&format!(": {}", ret_type.to_source()));
            }
            result.push_str(" {\n");
            for stmt in body {
                result.push_str(&format!("    {}\n", stmt.to_source()));
            }
            result.push_str("  }\n");
            result
        }
        ObjectMember::Event { name, params, body } => {
            let mut result = format!("  event {name}(");
            let params_str = params
                .iter()
                .map(|p| p.to_source())
                .collect::<Vec<_>>()
                .join(", ");
            result.push_str(&params_str);
            result.push_str(")\n");
            for stmt in body {
                result.push_str(&format!("    {}\n", stmt.to_source()));
            }
            result.push_str("  endevent\n");
            result
        }
        ObjectMember::Query {
            name,
            params,
            clauses,
        } => {
            let mut result = format!("  query {name}");
            if !params.is_empty() {
                result.push_str(&format!("({})", params.join(", ")));
            }
            result.push_str(" :-\n    ");
            let clauses_str = clauses
                .iter()
                .map(|c| {
                    let args_str = c
                        .args
                        .iter()
                        .map(|arg| match arg {
                            crate::ast::QueryArg::Variable(v) => v.clone(),
                            crate::ast::QueryArg::Constant(c) => c.to_source(),
                            crate::ast::QueryArg::Wildcard => "_".to_string(),
                        })
                        .collect::<Vec<_>>()
                        .join(", ");
                    format!("{}({})", c.predicate, args_str)
                })
                .collect::<Vec<_>>()
                .join(",\n    ");
            result.push_str(&clauses_str);
            result.push_str(".\n");
            result
        }
    }
}

impl ToSource for EchoAst {
    fn to_source(&self) -> String {
        match self {
            // Literals
            EchoAst::Number(n) => n.to_string(),
            EchoAst::Float(f) => f.to_string(),
            EchoAst::String(s) => format!("\"{}\"", escape_string(s)),
            EchoAst::Boolean(b) => b.to_string(),
            EchoAst::Null => "null".to_string(),

            // Identifiers and references
            EchoAst::Identifier(s) => s.clone(),
            EchoAst::SystemProperty(prop) => format!("${prop}"),
            EchoAst::ObjectRef(n) => format!("#{n}"),

            // Arithmetic operations
            EchoAst::Add { left, right } => format!("{} + {}", left.to_source(), right.to_source()),
            EchoAst::Subtract { left, right } => {
                format!("{} - {}", left.to_source(), right.to_source())
            }
            EchoAst::Multiply { left, right } => {
                format!("{} * {}", left.to_source(), right.to_source())
            }
            EchoAst::Divide { left, right } => {
                format!("{} / {}", left.to_source(), right.to_source())
            }
            EchoAst::Modulo { left, right } => {
                format!("{} % {}", left.to_source(), right.to_source())
            }
            EchoAst::Power { left, right } => {
                format!("{} ^ {}", left.to_source(), right.to_source())
            }

            // Comparison operations
            EchoAst::Equal { left, right } => {
                format!("{} == {}", left.to_source(), right.to_source())
            }
            EchoAst::NotEqual { left, right } => {
                format!("{} != {}", left.to_source(), right.to_source())
            }
            EchoAst::LessThan { left, right } => {
                format!("{} < {}", left.to_source(), right.to_source())
            }
            EchoAst::LessEqual { left, right } => {
                format!("{} <= {}", left.to_source(), right.to_source())
            }
            EchoAst::GreaterThan { left, right } => {
                format!("{} > {}", left.to_source(), right.to_source())
            }
            EchoAst::GreaterEqual { left, right } => {
                format!("{} >= {}", left.to_source(), right.to_source())
            }

            // Logical operations
            EchoAst::And { left, right } => {
                format!("{} && {}", left.to_source(), right.to_source())
            }
            EchoAst::Or { left, right } => format!("{} || {}", left.to_source(), right.to_source()),
            EchoAst::Not { operand } => format!("!{}", operand.to_source()),

            // Unary operations
            EchoAst::UnaryMinus { operand } => format!("-{}", operand.to_source()),
            EchoAst::UnaryPlus { operand } => format!("+{}", operand.to_source()),

            // Property access and method calls
            EchoAst::PropertyAccess { object, property } => {
                format!("{}.{}", object.to_source(), property)
            }
            EchoAst::MethodCall {
                object,
                method,
                args,
            } => {
                let args_str = args
                    .iter()
                    .map(|arg| arg.to_source())
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{}:{}({})", object.to_source(), method, args_str)
            }
            EchoAst::FunctionCall { name, args } => {
                let args_str = args
                    .iter()
                    .map(|arg| arg.to_source())
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{name}({args_str})")
            }
            EchoAst::Call { func, args } => {
                let args_str = args
                    .iter()
                    .map(|arg| arg.to_source())
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{}({})", func.to_source(), args_str)
            }

            // Index access
            EchoAst::IndexAccess { object, index } => {
                format!("{}[{}]", object.to_source(), index.to_source())
            }

            // Lists and Maps
            EchoAst::List { elements } => {
                let elements_str = elements
                    .iter()
                    .map(|elem| elem.to_source())
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("[{elements_str}]")
            }
            EchoAst::Map { entries } => {
                let entries_str = entries
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, v.to_source()))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{{{entries_str}}}")
            }

            // Control flow
            EchoAst::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let mut result = format!("if ({}) ", condition.to_source());
                for stmt in then_branch {
                    result.push_str(&format!("{} ", stmt.to_source()));
                }
                if let Some(else_stmts) = else_branch {
                    result.push_str("else ");
                    for stmt in else_stmts {
                        result.push_str(&format!("{} ", stmt.to_source()));
                    }
                }
                result.push_str("endif");
                result
            }

            EchoAst::While {
                label,
                condition,
                body,
            } => {
                let mut result = String::new();
                if let Some(lbl) = label {
                    result.push_str(&format!("{lbl}: "));
                }
                result.push_str(&format!("while ({}) ", condition.to_source()));
                for stmt in body {
                    result.push_str(&format!("{} ", stmt.to_source()));
                }
                result.push_str("endwhile");
                result
            }

            EchoAst::For {
                label,
                variable,
                collection,
                body,
            } => {
                let mut result = String::new();
                if let Some(lbl) = label {
                    result.push_str(&format!("{lbl}: "));
                }
                result.push_str(&format!(
                    "for ({} in {}) ",
                    variable,
                    collection.to_source()
                ));
                for stmt in body {
                    result.push_str(&format!("{} ", stmt.to_source()));
                }
                result.push_str("endfor");
                result
            }

            EchoAst::Break { label } => {
                if let Some(lbl) = label {
                    format!("break {lbl}")
                } else {
                    "break".to_string()
                }
            }

            EchoAst::Continue { label } => {
                if let Some(lbl) = label {
                    format!("continue {lbl}")
                } else {
                    "continue".to_string()
                }
            }

            EchoAst::Return { value } => {
                if let Some(val) = value {
                    format!("return {}", val.to_source())
                } else {
                    "return".to_string()
                }
            }

            EchoAst::Emit { event_name, args } => {
                if args.is_empty() {
                    format!("emit {event_name}")
                } else {
                    let args_str = args
                        .iter()
                        .map(|arg| arg.to_source())
                        .collect::<Vec<_>>()
                        .join(", ");
                    format!("emit {event_name}({args_str})")
                }
            }

            // Object definition
            EchoAst::ObjectDef {
                name,
                parent,
                members,
            } => {
                let mut result = format!("object {name}");
                if let Some(p) = parent {
                    result.push_str(&format!(" extends {p}"));
                }
                result.push('\n');

                for member in members {
                    result.push_str(&format_object_member(member));
                }

                result.push_str("endobject");
                result
            }

            // Lambda functions
            EchoAst::Lambda { params, body } => {
                let params_str = params
                    .iter()
                    .map(|p| p.to_source())
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("fn {{{}}} {} endfn", params_str, body.to_source())
            }

            // Assignment
            EchoAst::Assignment { target, value } => {
                format!("{} = {}", target.to_source(), value.to_source())
            }
            EchoAst::LocalAssignment { target, value } => {
                format!("let {} = {}", target.to_source(), value.to_source())
            }
            EchoAst::ConstAssignment { target, value } => {
                format!("const {} = {}", target.to_source(), value.to_source())
            }

            // Error handling
            EchoAst::Try {
                body,
                catch,
                finally,
            } => {
                let mut result = "try\n".to_string();
                for stmt in body {
                    result.push_str(&format!("  {}\n", stmt.to_source()));
                }
                if let Some(catch_clause) = catch {
                    result.push_str("catch");
                    if let Some(var) = &catch_clause.error_var {
                        result.push_str(&format!(" ({var})"));
                    }
                    result.push('\n');
                    for stmt in &catch_clause.body {
                        result.push_str(&format!("  {}\n", stmt.to_source()));
                    }
                }
                if let Some(finally_body) = finally {
                    result.push_str("finally\n");
                    for stmt in finally_body {
                        result.push_str(&format!("  {}\n", stmt.to_source()));
                    }
                }
                result.push_str("endtry");
                result
            }

            // Blocks and statements
            EchoAst::Block(statements) => {
                let mut result = "{\n".to_string();
                for stmt in statements {
                    result.push_str(&format!("  {}\n", stmt.to_source()));
                }
                result.push('}');
                result
            }
            EchoAst::ExpressionStatement(expr) => expr.to_source(),

            // Program (sequence of statements)
            EchoAst::Program(statements) => statements
                .iter()
                .map(|stmt| stmt.to_source())
                .collect::<Vec<_>>()
                .join("\n"),

            // Modern Echo features
            EchoAst::Event { name, params, body } => {
                let params_str = params
                    .iter()
                    .map(|p| p.to_source())
                    .collect::<Vec<_>>()
                    .join(", ");
                let mut result = format!("event {name}({params_str})\n");
                for stmt in body {
                    result.push_str(&format!("  {}\n", stmt.to_source()));
                }
                result.push_str("endevent");
                result
            }

            EchoAst::Spawn { body } => format!("spawn {}", body.to_source()),
            EchoAst::Await { expr } => format!("await {}", expr.to_source()),

            EchoAst::Match { expr, arms } => {
                let mut result = format!("match {} {{\n", expr.to_source());
                for arm in arms {
                    result.push_str(&format!(
                        "  {} => {},\n",
                        arm.pattern.to_source(),
                        arm.body.to_source()
                    ));
                }
                result.push('}');
                result
            }

            EchoAst::TypedIdentifier {
                name,
                type_annotation,
            } => {
                format!("{}: {}", name, type_annotation.to_source())
            }

            EchoAst::In { left, right } => {
                format!("{} in {}", left.to_source(), right.to_source())
            }

            // MOO-specific features
            EchoAst::Flyweight { object, properties } => {
                let props = properties
                    .iter()
                    .map(|(k, v)| format!("{} -> {}", k, v.to_source()))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("<{}, [{}]>", object.to_source(), props)
            }
            
            EchoAst::ErrorCatch { expr, error_patterns, default } => {
                let patterns = error_patterns.join(", ");
                format!("`{} ! {} => {}'", expr.to_source(), patterns, default.to_source())
            }
            
            EchoAst::MapLiteral { entries } => {
                let items = entries
                    .iter()
                    .map(|(k, v)| format!("{} -> {}", k, v.to_source()))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("[{}]", items)
            }
            
            EchoAst::Define { name, value } => {
                format!("define {} = {}", name, value.to_source())
            }
            
            EchoAst::Spread { expr } => {
                format!("...{}", expr.to_source())
            }
            
            EchoAst::DestructuringAssignment { targets, value } => {
                let target_list = targets
                    .iter()
                    .map(|t| t.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{{{}}} = {}", target_list, value.to_source())
            }
        }
    }
}

impl ToSource for LValue {
    fn to_source(&self) -> String {
        match self {
            LValue::Binding {
                binding_type,
                pattern,
            } => {
                let type_str = match binding_type {
                    BindingType::Let => "let",
                    BindingType::Const => "const",
                    BindingType::None => "",
                };
                if type_str.is_empty() {
                    pattern.to_source()
                } else {
                    format!("{} {}", type_str, pattern.to_source())
                }
            }
            LValue::PropertyAccess { object, property } => {
                format!("{}.{}", object.to_source(), property)
            }
            LValue::IndexAccess { object, index } => {
                format!("{}[{}]", object.to_source(), index.to_source())
            }
        }
    }
}

impl ToSource for BindingPattern {
    fn to_source(&self) -> String {
        match self {
            BindingPattern::Identifier(name) => name.clone(),
            BindingPattern::List(elements) => {
                let elements_str = elements
                    .iter()
                    .map(|elem| elem.to_source())
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{{{elements_str}}}")
            }
            BindingPattern::Object(entries) => {
                let entries_str = entries
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, v.to_source()))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{{{entries_str}}}")
            }
            BindingPattern::Rest(pattern) => format!("...{}", pattern.to_source()),
            BindingPattern::Ignore => "_".to_string(),
        }
    }
}

impl ToSource for BindingPatternElement {
    fn to_source(&self) -> String {
        match self {
            BindingPatternElement::Simple(name) => name.clone(),
            BindingPatternElement::Optional { name, default } => {
                format!("?{} = {}", name, default.to_source())
            }
            BindingPatternElement::Rest(name) => format!("@{name}"),
        }
    }
}

impl ToSource for Parameter {
    fn to_source(&self) -> String {
        let mut result = self.name.clone();
        if let Some(type_ann) = &self.type_annotation {
            result.push_str(&format!(": {}", type_ann.to_source()));
        }
        if let Some(default) = &self.default_value {
            result.push_str(&format!(" = {}", default.to_source()));
        }
        result
    }
}

impl ToSource for LambdaParam {
    fn to_source(&self) -> String {
        match self {
            LambdaParam::Simple(name) => name.clone(),
            LambdaParam::Optional { name, default } => {
                format!("?{} = {}", name, default.to_source())
            }
            LambdaParam::Rest(name) => format!("@{name}"),
        }
    }
}

impl ToSource for Pattern {
    fn to_source(&self) -> String {
        match self {
            Pattern::Wildcard => "_".to_string(),
            Pattern::Identifier(name) => name.clone(),
            Pattern::Number(n) => n.to_string(),
            Pattern::String(s) => format!("\"{}\"", escape_string(s)),
            Pattern::Constructor { name, args } => {
                let args_str = args
                    .iter()
                    .map(|arg| arg.to_source())
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{name}({args_str})")
            }
        }
    }
}

impl ToSource for TypeExpression {
    fn to_source(&self) -> String {
        match self {
            TypeExpression::Named(name) => name.clone(),
            TypeExpression::Array(inner) => format!("{}[]", inner.to_source()),
            TypeExpression::Optional(inner) => format!("{}?", inner.to_source()),
            TypeExpression::Function {
                params,
                return_type,
            } => {
                let params_str = params
                    .iter()
                    .map(|p| p.to_source())
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("({}) -> {}", params_str, return_type.to_source())
            }
        }
    }
}

// Helper function to escape string characters
fn escape_string(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '"' => r#"\""#.to_string(),
            '\\' => r"\\".to_string(),
            '\n' => r"\n".to_string(),
            '\r' => r"\r".to_string(),
            '\t' => r"\t".to_string(),
            c => c.to_string(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_expressions() {
        let num = EchoAst::Number(42);
        assert_eq!(num.to_source(), "42");

        let float = EchoAst::Float(std::f64::consts::PI);
        assert_eq!(float.to_source(), std::f64::consts::PI.to_string());

        let bool_val = EchoAst::Boolean(true);
        assert_eq!(bool_val.to_source(), "true");

        let string = EchoAst::String("hello".to_string());
        assert_eq!(string.to_source(), "\"hello\"");
    }

    #[test]
    fn test_arithmetic() {
        let add = EchoAst::Add {
            left: Box::new(EchoAst::Number(2)),
            right: Box::new(EchoAst::Number(3)),
        };
        assert_eq!(add.to_source(), "2 + 3");
    }

    #[test]
    fn test_if_statement() {
        let if_stmt = EchoAst::If {
            condition: Box::new(EchoAst::GreaterThan {
                left: Box::new(EchoAst::Identifier("x".to_string())),
                right: Box::new(EchoAst::Number(5)),
            }),
            then_branch: vec![EchoAst::Assignment {
                target: LValue::Binding {
                    binding_type: BindingType::None,
                    pattern: BindingPattern::Identifier("x".to_string()),
                },
                value: Box::new(EchoAst::Number(10)),
            }],
            else_branch: None,
        };
        let source = if_stmt.to_source();
        assert!(source.contains("if (x > 5)"));
        assert!(source.contains("x = 10"));
        assert!(source.contains("endif"));
    }

    #[test]
    fn test_lambda() {
        let lambda = EchoAst::Lambda {
            params: vec![
                LambdaParam::Simple("x".to_string()),
                LambdaParam::Simple("y".to_string()),
            ],
            body: Box::new(EchoAst::Add {
                left: Box::new(EchoAst::Identifier("x".to_string())),
                right: Box::new(EchoAst::Identifier("y".to_string())),
            }),
        };
        let source = lambda.to_source();
        assert!(source.contains("fn {x, y}"));
        assert!(source.contains("x + y"));
        assert!(source.contains("endfn"));
    }

    #[test]
    fn test_escaped_strings() {
        let string = EchoAst::String("Hello \"world\"\nNew line".to_string());
        assert_eq!(string.to_source(), r#""Hello \"world\"\nNew line""#);
    }
}
