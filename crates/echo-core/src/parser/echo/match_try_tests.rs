//! Tests for Match and Try expressions in rust-sitter grammar

#[cfg(test)]
mod tests {
    use crate::parser::create_parser;
    use crate::ast::{EchoAst, Pattern};

    #[test]
    fn test_match_expression_numbers() {
        let input = r#"
match 42
case 10 => "ten"
case 42 => "forty-two"  
case _ => "other"
endmatch
        "#;
        
        let mut parser = create_parser("echo").unwrap();
        let result = parser.parse(input).unwrap();
        
        match result {
            EchoAst::Match { expr, arms } => {
                // Check expression
                assert!(matches!(expr.as_ref(), EchoAst::Number(42)));
                
                // Check arms
                assert_eq!(arms.len(), 3);
                
                // First arm: case 10
                assert!(matches!(arms[0].pattern, Pattern::Number(10)));
                assert!(arms[0].guard.is_none());
                assert!(matches!(arms[0].body.as_ref(), EchoAst::String(s) if s == "ten"));
                
                // Second arm: case 42
                assert!(matches!(arms[1].pattern, Pattern::Number(42)));
                assert!(arms[1].guard.is_none());
                assert!(matches!(arms[1].body.as_ref(), EchoAst::String(s) if s == "forty-two"));
                
                // Third arm: wildcard
                assert!(matches!(arms[2].pattern, Pattern::Wildcard));
                assert!(arms[2].guard.is_none());
                assert!(matches!(arms[2].body.as_ref(), EchoAst::String(s) if s == "other"));
            }
            _ => panic!("Expected Match expression, got {:?}", result),
        }
    }

    #[test]
    fn test_match_with_string_patterns() {
        let input = r#"
match "hello"
case "world" => 1
case "hello" => 2
case _ => 3
endmatch
        "#;
        
        let mut parser = create_parser("echo").unwrap();
        let result = parser.parse(input).unwrap();
        
        match result {
            EchoAst::Match { expr, arms } => {
                assert!(matches!(expr.as_ref(), EchoAst::String(s) if s == "hello"));
                assert_eq!(arms.len(), 3);
                
                assert!(matches!(&arms[0].pattern, Pattern::String(s) if s == "world"));
                assert!(matches!(&arms[1].pattern, Pattern::String(s) if s == "hello"));
                assert!(matches!(arms[2].pattern, Pattern::Wildcard));
            }
            _ => panic!("Expected Match expression"),
        }
    }

    #[test]
    fn test_match_with_identifier_pattern() {
        let input = r#"
match 42
case x => x + 10
endmatch
        "#;
        
        let mut parser = create_parser("echo").unwrap();
        let result = parser.parse(input).unwrap();
        
        match result {
            EchoAst::Match { expr, arms } => {
                assert!(matches!(expr.as_ref(), EchoAst::Number(42)));
                assert_eq!(arms.len(), 1);
                
                assert!(matches!(&arms[0].pattern, Pattern::Identifier(s) if s == "x"));
                assert!(arms[0].guard.is_none());
                // Body should be x + 10 (an Add expression)
                assert!(matches!(arms[0].body.as_ref(), EchoAst::Add { .. }));
            }
            _ => panic!("Expected Match expression"),
        }
    }

    #[test]
    fn test_match_with_guard() {
        let input = r#"
match 42
case x when x > 10 => "big"
case _ => "small"
endmatch
        "#;
        
        let mut parser = create_parser("echo").unwrap();
        let result = parser.parse(input).unwrap();
        
        match result {
            EchoAst::Match { expr: _, arms } => {
                assert_eq!(arms.len(), 2);
                
                // First arm with guard
                assert!(matches!(&arms[0].pattern, Pattern::Identifier(s) if s == "x"));
                assert!(arms[0].guard.is_some());
                // Guard should be x > 10 (a GreaterThan expression)
                assert!(matches!(arms[0].guard.as_ref().unwrap().as_ref(), EchoAst::GreaterThan { .. }));
                
                // Second arm without guard
                assert!(matches!(arms[1].pattern, Pattern::Wildcard));
                assert!(arms[1].guard.is_none());
            }
            _ => panic!("Expected Match expression"),
        }
    }

    #[test]
    fn test_try_catch_expression() {
        let input = r#"
try
    42
catch e
    "error occurred"
endtry
        "#;
        
        let mut parser = create_parser("echo").unwrap();
        let result = parser.parse(input).unwrap();
        
        match result {
            EchoAst::Try { body, catch, finally } => {
                // Check try body
                assert_eq!(body.len(), 1);
                assert!(matches!(body[0], EchoAst::Number(42)));
                
                // Check catch clause
                assert!(catch.is_some());
                let catch_clause = catch.unwrap();
                assert_eq!(catch_clause.error_var, Some("e".to_string()));
                assert_eq!(catch_clause.body.len(), 1);
                assert!(matches!(&catch_clause.body[0], EchoAst::String(s) if s == "error occurred"));
                
                // Check finally clause
                assert!(finally.is_none());
            }
            _ => panic!("Expected Try expression"),
        }
    }

    #[test]
    fn test_try_catch_finally_expression() {
        let input = r#"
try
    100
catch err
    -1
finally
    "cleanup"
endtry
        "#;
        
        let mut parser = create_parser("echo").unwrap();
        let result = parser.parse(input).unwrap();
        
        match result {
            EchoAst::Try { body, catch, finally } => {
                // Check try body
                assert_eq!(body.len(), 1);
                assert!(matches!(body[0], EchoAst::Number(100)));
                
                // Check catch clause
                assert!(catch.is_some());
                let catch_clause = catch.unwrap();
                assert_eq!(catch_clause.error_var, Some("err".to_string()));
                assert_eq!(catch_clause.body.len(), 1);
                // -1 should be parsed as Number(-1)
                assert!(matches!(catch_clause.body[0], EchoAst::Number(-1)));
                
                // Check finally clause
                assert!(finally.is_some());
                let finally_stmts = finally.unwrap();
                assert_eq!(finally_stmts.len(), 1);
                assert!(matches!(&finally_stmts[0], EchoAst::String(s) if s == "cleanup"));
            }
            _ => panic!("Expected Try expression"),
        }
    }

    #[test]
    fn test_nested_match_in_try() {
        let input = r#"
try
    match x
    case 1 => "one"
    case 2 => "two"
    case _ => "other"
    endmatch
catch e
    "error"
endtry
        "#;
        
        let mut parser = create_parser("echo").unwrap();
        let result = parser.parse(input).unwrap();
        
        match result {
            EchoAst::Try { body, catch, .. } => {
                assert_eq!(body.len(), 1);
                assert!(matches!(&body[0], EchoAst::Match { .. }));
                assert!(catch.is_some());
            }
            _ => panic!("Expected Try expression"),
        }
    }

    #[test]
    fn test_match_negative_numbers() {
        let input = r#"
match -42
case -10 => "negative ten"
case -42 => "negative forty-two"
case _ => "other"
endmatch
        "#;
        
        let mut parser = create_parser("echo").unwrap();
        let result = parser.parse(input).unwrap();
        
        match result {
            EchoAst::Match { expr, arms } => {
                // Check expression - should be Number(-42)
                assert!(matches!(expr.as_ref(), EchoAst::Number(-42)));
                
                // Check arms
                assert_eq!(arms.len(), 3);
                
                // First arm: case -10
                assert!(matches!(arms[0].pattern, Pattern::Number(-10)));
                
                // Second arm: case -42
                assert!(matches!(arms[1].pattern, Pattern::Number(-42)));
                
                // Third arm: wildcard
                assert!(matches!(arms[2].pattern, Pattern::Wildcard));
            }
            _ => panic!("Expected Match expression"),
        }
    }
}