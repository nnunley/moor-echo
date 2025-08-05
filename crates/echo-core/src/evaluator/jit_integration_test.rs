//! Integration test for Match and Try/Catch working together

#[cfg(all(test, feature = "jit"))]
mod tests {
    use super::super::*;
    use crate::ast::{EchoAst, MatchArm, Pattern, CatchClause};
    use crate::storage::Storage;
    use std::sync::Arc;
    use tempfile::TempDir;

    fn create_test_jit() -> JitEvaluator {
        let temp_dir = TempDir::new().unwrap();
        let storage = Arc::new(Storage::new(temp_dir.path()).unwrap());
        JitEvaluator::new_with_fallback(storage)
    }

    #[test]
    fn test_match_inside_try_catch() {
        let mut jit = create_test_jit();
        
        // Create a player
        let player_id = jit.create_player("test_player").unwrap();
        jit.switch_player(player_id).unwrap();
        
        // Test match expression inside try/catch
        let ast = EchoAst::Try {
            body: vec![
                EchoAst::Match {
                    expr: Box::new(EchoAst::Number(42)),
                    arms: vec![
                        MatchArm {
                            pattern: Pattern::Number(42),
                            guard: None,
                            body: Box::new(EchoAst::String("matched!".to_string())),
                        },
                        MatchArm {
                            pattern: Pattern::Wildcard,
                            guard: None,
                            body: Box::new(EchoAst::String("default".to_string())),
                        },
                    ],
                },
            ],
            catch: Some(CatchClause {
                error_var: Some("e".to_string()),
                body: vec![
                    EchoAst::String("error in match".to_string()),
                ],
            }),
            finally: Some(vec![
                EchoAst::String("cleanup".to_string()),
            ]),
        };
        
        let result = jit.eval(&ast).unwrap();
        
        match result {
            Value::String(s) => assert_eq!(s, "matched!"),
            _ => panic!("Expected string value"),
        }
    }

    #[test] 
    fn test_try_catch_inside_match() {
        let mut jit = create_test_jit();
        
        // Create a player
        let player_id = jit.create_player("test_player").unwrap();
        jit.switch_player(player_id).unwrap();
        
        // Test try/catch inside match arm
        let ast = EchoAst::Match {
            expr: Box::new(EchoAst::String("test".to_string())),
            arms: vec![
                MatchArm {
                    pattern: Pattern::String("test".to_string()),
                    guard: None,
                    body: Box::new(EchoAst::Try {
                        body: vec![
                            EchoAst::Number(100),
                        ],
                        catch: Some(CatchClause {
                            error_var: None,
                            body: vec![
                                EchoAst::Number(-1),
                            ],
                        }),
                        finally: None,
                    }),
                },
                MatchArm {
                    pattern: Pattern::Wildcard,
                    guard: None,
                    body: Box::new(EchoAst::Number(0)),
                },
            ],
        };
        
        let result = jit.eval(&ast).unwrap();
        
        match result {
            Value::Integer(n) => assert_eq!(n, 100),
            _ => panic!("Expected integer value"),
        }
    }

    #[test]
    fn test_match_with_variable_capture_in_try() {
        let mut jit = create_test_jit();
        
        // Create a player
        let player_id = jit.create_player("test_player").unwrap();
        jit.switch_player(player_id).unwrap();
        
        // Test match variable capture working with try/catch
        let ast = EchoAst::Try {
            body: vec![
                EchoAst::Match {
                    expr: Box::new(EchoAst::Number(99)),
                    arms: vec![
                        MatchArm {
                            pattern: Pattern::Identifier("captured_value".to_string()),
                            guard: None,
                            body: Box::new(EchoAst::Add {
                                left: Box::new(EchoAst::Identifier("captured_value".to_string())),
                                right: Box::new(EchoAst::Number(1)),
                            }),
                        },
                    ],
                },
            ],
            catch: None,
            finally: None,
        };
        
        let result = jit.eval(&ast).unwrap();
        
        match result {
            Value::Integer(n) => assert_eq!(n, 100),
            _ => panic!("Expected integer value"),
        }
    }
}