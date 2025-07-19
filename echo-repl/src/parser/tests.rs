#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{EchoAst, LValue, BindingType, BindingPattern};
    use crate::parser::{Parser, create_parser};
    
    fn parse(input: &str) -> Result<EchoAst, anyhow::Error> {
        let mut parser = create_parser("echo")?;
        parser.parse(input)
    }
    
    #[test]
    fn test_literals() {
        // Numbers
        assert!(matches!(parse("42").unwrap(), EchoAst::Number(42)));
        assert!(matches!(parse("0").unwrap(), EchoAst::Number(0)));
        assert!(matches!(parse("-42").unwrap(), EchoAst::Number(-42)));
        
        // Floats
        assert!(matches!(parse("3.14").unwrap(), EchoAst::Float(f) if (f - 3.14).abs() < f64::EPSILON));
        assert!(matches!(parse("0.0").unwrap(), EchoAst::Float(f) if f == 0.0));
        
        // Strings
        assert!(matches!(parse(r#""hello""#).unwrap(), EchoAst::String(s) if s == "hello"));
        assert!(matches!(parse(r#""""#).unwrap(), EchoAst::String(s) if s.is_empty()));
        
        // Booleans
        assert!(matches!(parse("true").unwrap(), EchoAst::Boolean(true)));
        assert!(matches!(parse("false").unwrap(), EchoAst::Boolean(false)));
    }
    
    #[test]
    fn test_identifiers_and_references() {
        // Identifiers
        assert!(matches!(parse("foo").unwrap(), EchoAst::Identifier(s) if s == "foo"));
        assert!(matches!(parse("_underscore").unwrap(), EchoAst::Identifier(s) if s == "_underscore"));
        assert!(matches!(parse("var123").unwrap(), EchoAst::Identifier(s) if s == "var123"));
        
        // System properties
        assert!(matches!(parse("$system").unwrap(), EchoAst::SystemProperty(s) if s == "system"));
        assert!(matches!(parse("$root").unwrap(), EchoAst::SystemProperty(s) if s == "root"));
        
        // Object references
        assert!(matches!(parse("#0").unwrap(), EchoAst::ObjectRef(0)));
        assert!(matches!(parse("#123").unwrap(), EchoAst::ObjectRef(123)));
    }
    
    #[test]
    fn test_arithmetic_operations() {
        // Addition
        let ast = parse("1 + 2").unwrap();
        assert!(matches!(ast, EchoAst::Add { .. }));
        
        // String concatenation
        let ast = parse(r#""hello" + "world""#).unwrap();
        assert!(matches!(ast, EchoAst::Add { .. }));
        
        // Subtraction
        assert!(matches!(parse("5 - 3").unwrap(), EchoAst::Subtract { .. }));
        
        // Multiplication
        assert!(matches!(parse("4 * 2").unwrap(), EchoAst::Multiply { .. }));
        
        // Division
        assert!(matches!(parse("10 / 2").unwrap(), EchoAst::Divide { .. }));
        
        // Modulo
        assert!(matches!(parse("10 % 3").unwrap(), EchoAst::Modulo { .. }));
        
        // Complex expressions
        let ast = parse("(1 + 2) * 3").unwrap();
        assert!(matches!(ast, EchoAst::Multiply { .. }));
    }
    
    #[test]
    fn test_comparison_operations() {
        assert!(matches!(parse("1 == 2").unwrap(), EchoAst::Equal { .. }));
        assert!(matches!(parse("1 != 2").unwrap(), EchoAst::NotEqual { .. }));
        assert!(matches!(parse("1 < 2").unwrap(), EchoAst::LessThan { .. }));
        assert!(matches!(parse("1 <= 2").unwrap(), EchoAst::LessEqual { .. }));
        assert!(matches!(parse("1 > 2").unwrap(), EchoAst::GreaterThan { .. }));
        assert!(matches!(parse("1 >= 2").unwrap(), EchoAst::GreaterEqual { .. }));
    }
    
    #[test]
    fn test_logical_operations() {
        assert!(matches!(parse("true && false").unwrap(), EchoAst::And { .. }));
        assert!(matches!(parse("true || false").unwrap(), EchoAst::Or { .. }));
        assert!(matches!(parse("!true").unwrap(), EchoAst::Not { .. }));
        
        // Complex logical expressions
        let ast = parse("(a && b) || !c").unwrap();
        assert!(matches!(ast, EchoAst::Or { .. }));
    }
    
    #[test]
    fn test_simple_assignment() {
        let ast = parse("x = 42").unwrap();
        match ast {
            EchoAst::Assignment { target, value } => {
                match target {
                    LValue::Binding { binding_type, pattern } => {
                        assert_eq!(binding_type, BindingType::None);
                        assert!(matches!(pattern, BindingPattern::Identifier(s) if s == "x"));
                    }
                    _ => panic!("Expected binding LValue"),
                }
                assert!(matches!(**value, EchoAst::Number(42)));
            }
            _ => panic!("Expected assignment"),
        }
    }
    
    #[test]
    fn test_let_binding() {
        let ast = parse("let x = 42").unwrap();
        match ast {
            EchoAst::Assignment { target, value } => {
                match target {
                    LValue::Binding { binding_type, pattern } => {
                        assert_eq!(binding_type, BindingType::Let);
                        assert!(matches!(pattern, BindingPattern::Identifier(s) if s == "x"));
                    }
                    _ => panic!("Expected binding LValue"),
                }
                assert!(matches!(**value, EchoAst::Number(42)));
            }
            _ => panic!("Expected assignment"),
        }
    }
    
    #[test]
    fn test_const_binding() {
        let ast = parse("const PI = 3.14").unwrap();
        match ast {
            EchoAst::Assignment { target, value } => {
                match target {
                    LValue::Binding { binding_type, pattern } => {
                        assert_eq!(binding_type, BindingType::Const);
                        assert!(matches!(pattern, BindingPattern::Identifier(s) if s == "PI"));
                    }
                    _ => panic!("Expected binding LValue"),
                }
                assert!(matches!(**value, EchoAst::Float(_)));
            }
            _ => panic!("Expected assignment"),
        }
    }
    
    #[test]
    fn test_property_assignment() {
        let ast = parse("obj.prop = 123").unwrap();
        match ast {
            EchoAst::Assignment { target, value } => {
                match target {
                    LValue::PropertyAccess { object, property } => {
                        assert!(matches!(**object, EchoAst::Identifier(s) if s == "obj"));
                        assert_eq!(property, "prop");
                    }
                    _ => panic!("Expected property access LValue"),
                }
                assert!(matches!(**value, EchoAst::Number(123)));
            }
            _ => panic!("Expected assignment"),
        }
    }
    
    #[test]
    fn test_property_access() {
        let ast = parse("obj.prop").unwrap();
        match ast {
            EchoAst::PropertyAccess { object, property } => {
                assert!(matches!(**object, EchoAst::Identifier(s) if s == "obj"));
                assert_eq!(property, "prop");
            }
            _ => panic!("Expected property access"),
        }
        
        // Chained property access
        let ast = parse("obj.prop.subprop").unwrap();
        match ast {
            EchoAst::PropertyAccess { object, property } => {
                assert_eq!(property, "subprop");
                assert!(matches!(**object, EchoAst::PropertyAccess { .. }));
            }
            _ => panic!("Expected property access"),
        }
    }
    
    #[test]
    fn test_lists() {
        // Empty list
        let ast = parse("[]").unwrap();
        match ast {
            EchoAst::List { elements } => {
                assert!(elements.is_empty());
            }
            _ => panic!("Expected list"),
        }
        
        // Single element
        let ast = parse("[42]").unwrap();
        match ast {
            EchoAst::List { elements } => {
                assert_eq!(elements.len(), 1);
                assert!(matches!(elements[0], EchoAst::Number(42)));
            }
            _ => panic!("Expected list"),
        }
        
        // Multiple elements
        let ast = parse("[1, 2, 3]").unwrap();
        match ast {
            EchoAst::List { elements } => {
                assert_eq!(elements.len(), 3);
            }
            _ => panic!("Expected list"),
        }
    }
    
    #[test]
    fn test_lambda_expressions() {
        // Simple lambda
        let ast = parse(r#"x => x + 1"#).unwrap();
        assert!(matches!(ast, EchoAst::Lambda { .. }));
        
        // Multi-parameter lambda
        let ast = parse(r#"(x, y) => x + y"#).unwrap();
        assert!(matches!(ast, EchoAst::Lambda { .. }));
        
        // Lambda with optional parameter
        let ast = parse(r#"(x, y = 10) => x + y"#).unwrap();
        assert!(matches!(ast, EchoAst::Lambda { .. }));
    }
    
    #[test]
    fn test_function_calls() {
        // Simple call
        let ast = parse("foo()").unwrap();
        assert!(matches!(ast, EchoAst::Call { .. }));
        
        // Call with arguments
        let ast = parse("foo(1, 2, 3)").unwrap();
        match ast {
            EchoAst::Call { func: _, args } => {
                assert_eq!(args.len(), 3);
            }
            _ => panic!("Expected call"),
        }
        
        // Method call
        let ast = parse("obj.method(42)").unwrap();
        assert!(matches!(ast, EchoAst::MethodCall { .. }));
    }
    
    #[test]
    fn test_control_flow() {
        // If statement
        let ast = parse("if x > 0 then x else -x endif").unwrap();
        assert!(matches!(ast, EchoAst::If { .. }));
        
        // While loop
        let ast = parse("while x < 10 do x = x + 1 endwhile").unwrap();
        assert!(matches!(ast, EchoAst::While { .. }));
        
        // For loop
        let ast = parse("for x in [1, 2, 3] do print(x) endfor").unwrap();
        assert!(matches!(ast, EchoAst::For { .. }));
    }
    
    #[test]
    fn test_edge_cases() {
        // Complex nested expression
        let ast = parse("((a + b) * (c - d)) / (e + f)").unwrap();
        assert!(matches!(ast, EchoAst::Divide { .. }));
        
        // Mixed numeric types
        let ast = parse("1 + 2.5").unwrap();
        assert!(matches!(ast, EchoAst::Add { .. }));
        
        // Property access on object reference
        let ast = parse("#0.name").unwrap();
        assert!(matches!(ast, EchoAst::PropertyAccess { .. }));
    }
    
    #[test]
    fn test_parse_errors() {
        // Invalid syntax
        assert!(parse("let = 42").is_err());
        assert!(parse("42 +").is_err());
        assert!(parse("if x").is_err());
        
        // Invalid identifiers
        assert!(parse("123abc").is_err());
        
        // Unclosed strings
        assert!(parse(r#""unclosed"#).is_err());
    }
    
    #[test]
    fn test_operator_precedence() {
        // Multiplication before addition
        let ast = parse("1 + 2 * 3").unwrap();
        match ast {
            EchoAst::Add { left, right } => {
                assert!(matches!(**left, EchoAst::Number(1)));
                assert!(matches!(**right, EchoAst::Multiply { .. }));
            }
            _ => panic!("Expected addition with multiplication on right"),
        }
        
        // Comparison has lower precedence than arithmetic
        let ast = parse("1 + 2 < 3 * 4").unwrap();
        match ast {
            EchoAst::LessThan { left, right } => {
                assert!(matches!(**left, EchoAst::Add { .. }));
                assert!(matches!(**right, EchoAst::Multiply { .. }));
            }
            _ => panic!("Expected less than comparison"),
        }
    }
}