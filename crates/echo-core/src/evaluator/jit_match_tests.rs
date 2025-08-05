//! Tests for JIT compilation of Match expressions

#[cfg(all(test, feature = "jit"))]
mod tests {
    use super::super::*;
    use crate::ast::{EchoAst, MatchArm, Pattern};
    use crate::storage::Storage;
    use std::sync::Arc;
    use tempfile::TempDir;

    fn create_test_jit() -> JitEvaluator {
        let temp_dir = TempDir::new().unwrap();
        let storage = Arc::new(Storage::new(temp_dir.path()).unwrap());
        JitEvaluator::new_with_fallback(storage)
    }

    #[test]
    fn test_jit_match_number_pattern() {
        let mut jit = create_test_jit();
        
        // Create a player
        let player_id = jit.create_player("test_player").unwrap();
        jit.switch_player(player_id).unwrap();
        
        // Test match with number patterns
        let ast = EchoAst::Match {
            expr: Box::new(EchoAst::Number(42)),
            arms: vec![
                MatchArm {
                    pattern: Pattern::Number(10),
                    guard: None,
                    body: Box::new(EchoAst::String("ten".to_string())),
                },
                MatchArm {
                    pattern: Pattern::Number(42),
                    guard: None,
                    body: Box::new(EchoAst::String("forty-two".to_string())),
                },
                MatchArm {
                    pattern: Pattern::Wildcard,
                    guard: None,
                    body: Box::new(EchoAst::String("other".to_string())),
                },
            ],
        };
        
        let result = jit.eval(&ast).unwrap();
        
        match result {
            Value::String(s) => assert_eq!(s, "forty-two"),
            _ => panic!("Expected string value"),
        }
    }

    #[test]
    fn test_jit_match_string_pattern() {
        let mut jit = create_test_jit();
        
        // Create a player
        let player_id = jit.create_player("test_player").unwrap();
        jit.switch_player(player_id).unwrap();
        
        // Test match with string patterns
        let ast = EchoAst::Match {
            expr: Box::new(EchoAst::String("hello".to_string())),
            arms: vec![
                MatchArm {
                    pattern: Pattern::String("world".to_string()),
                    guard: None,
                    body: Box::new(EchoAst::Number(1)),
                },
                MatchArm {
                    pattern: Pattern::String("hello".to_string()),
                    guard: None,
                    body: Box::new(EchoAst::Number(2)),
                },
                MatchArm {
                    pattern: Pattern::Wildcard,
                    guard: None,
                    body: Box::new(EchoAst::Number(3)),
                },
            ],
        };
        
        let result = jit.eval(&ast).unwrap();
        
        match result {
            Value::Integer(n) => assert_eq!(n, 2),
            _ => panic!("Expected integer value"),
        }
    }

    #[test]
    fn test_jit_match_wildcard_pattern() {
        let mut jit = create_test_jit();
        
        // Create a player
        let player_id = jit.create_player("test_player").unwrap();
        jit.switch_player(player_id).unwrap();
        
        // Test match with wildcard pattern
        let ast = EchoAst::Match {
            expr: Box::new(EchoAst::Number(999)),
            arms: vec![
                MatchArm {
                    pattern: Pattern::Number(10),
                    guard: None,
                    body: Box::new(EchoAst::String("ten".to_string())),
                },
                MatchArm {
                    pattern: Pattern::Wildcard,
                    guard: None,
                    body: Box::new(EchoAst::String("anything".to_string())),
                },
            ],
        };
        
        let result = jit.eval(&ast).unwrap();
        
        match result {
            Value::String(s) => assert_eq!(s, "anything"),
            _ => panic!("Expected string value"),
        }
    }

    #[test]
    fn test_jit_match_with_guard() {
        let mut jit = create_test_jit();
        
        // Create a player
        let player_id = jit.create_player("test_player").unwrap();
        jit.switch_player(player_id).unwrap();
        
        // Test match with guard condition
        let ast = EchoAst::Match {
            expr: Box::new(EchoAst::Number(42)),
            arms: vec![
                MatchArm {
                    pattern: Pattern::Number(42),
                    guard: Some(Box::new(EchoAst::Boolean(false))), // Guard fails
                    body: Box::new(EchoAst::String("guarded".to_string())),
                },
                MatchArm {
                    pattern: Pattern::Number(42),
                    guard: None,
                    body: Box::new(EchoAst::String("unguarded".to_string())),
                },
            ],
        };
        
        let result = jit.eval(&ast).unwrap();
        
        match result {
            Value::String(s) => assert_eq!(s, "unguarded"),
            _ => panic!("Expected string value"),
        }
    }

    #[test]
    fn test_jit_match_identifier_pattern() {
        let mut jit = create_test_jit();
        
        // Create a player
        let player_id = jit.create_player("test_player").unwrap();
        jit.switch_player(player_id).unwrap();
        
        // Test match with identifier pattern (captures value)
        let ast = EchoAst::Match {
            expr: Box::new(EchoAst::Number(42)),
            arms: vec![
                MatchArm {
                    pattern: Pattern::Identifier("x".to_string()),
                    guard: None,
                    body: Box::new(EchoAst::Add {
                        left: Box::new(EchoAst::Identifier("x".to_string())),
                        right: Box::new(EchoAst::Number(10)),
                    }),
                },
            ],
        };
        
        let result = jit.eval(&ast).unwrap();
        
        match result {
            Value::Integer(n) => assert_eq!(n, 52),
            _ => panic!("Expected integer value"),
        }
    }

    #[test]
    fn test_jit_compile_match() {
        let mut jit = create_test_jit();
        
        // Skip test if JIT is not enabled
        if !jit.is_jit_enabled() {
            println!("Skipping match compilation test - JIT not enabled");
            return;
        }
        
        // Test that compile_ast handles match expressions
        let match_expressions = vec![
            EchoAst::Match {
                expr: Box::new(EchoAst::Number(42)),
                arms: vec![
                    MatchArm {
                        pattern: Pattern::Number(42),
                        guard: None,
                        body: Box::new(EchoAst::String("matched".to_string())),
                    },
                ],
            },
            EchoAst::Match {
                expr: Box::new(EchoAst::String("test".to_string())),
                arms: vec![
                    MatchArm {
                        pattern: Pattern::Wildcard,
                        guard: None,
                        body: Box::new(EchoAst::Number(1)),
                    },
                ],
            },
        ];
        
        for expr in match_expressions {
            match jit.compile_ast(&expr) {
                Ok(()) => println!("Successfully compiled: {:?}", expr),
                Err(e) => {
                    if e.to_string().contains("falling back to interpreter") {
                        println!("Match falls back to interpreter as expected: {:?}", expr);
                    } else {
                        panic!("Failed to compile {:?}: {}", expr, e);
                    }
                }
            }
        }
    }
}