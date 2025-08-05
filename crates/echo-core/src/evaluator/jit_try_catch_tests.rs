//! Tests for JIT compilation of Try/Catch expressions

#[cfg(all(test, feature = "jit"))]
mod tests {
    use super::super::*;
    use crate::ast::{EchoAst, CatchClause};
    use crate::storage::Storage;
    use std::sync::Arc;
    use tempfile::TempDir;

    fn create_test_jit() -> JitEvaluator {
        let temp_dir = TempDir::new().unwrap();
        let storage = Arc::new(Storage::new(temp_dir.path()).unwrap());
        JitEvaluator::new_with_fallback(storage)
    }

    #[test]
    fn test_jit_try_no_error() {
        let mut jit = create_test_jit();
        
        // Create a player
        let player_id = jit.create_player("test_player").unwrap();
        jit.switch_player(player_id).unwrap();
        
        // Test try block with no error
        let ast = EchoAst::Try {
            body: vec![
                EchoAst::Number(42),
            ],
            catch: Some(CatchClause {
                error_var: Some("e".to_string()),
                body: vec![
                    EchoAst::String("error occurred".to_string()),
                ],
            }),
            finally: None,
        };
        
        let result = jit.eval(&ast).unwrap();
        
        match result {
            Value::Integer(n) => assert_eq!(n, 42),
            _ => panic!("Expected integer value"),
        }
    }

    #[test]
    fn test_jit_try_with_finally() {
        let mut jit = create_test_jit();
        
        // Create a player
        let player_id = jit.create_player("test_player").unwrap();
        jit.switch_player(player_id).unwrap();
        
        // Test try block with finally
        let ast = EchoAst::Try {
            body: vec![
                EchoAst::Number(42),
            ],
            catch: None,
            finally: Some(vec![
                EchoAst::String("cleanup".to_string()),
            ]),
        };
        
        let result = jit.eval(&ast).unwrap();
        
        match result {
            Value::Integer(n) => assert_eq!(n, 42),
            _ => panic!("Expected integer value"),
        }
    }

    #[test]
    fn test_jit_try_catch_finally() {
        let mut jit = create_test_jit();
        
        // Create a player
        let player_id = jit.create_player("test_player").unwrap();
        jit.switch_player(player_id).unwrap();
        
        // Test try block with both catch and finally
        let ast = EchoAst::Try {
            body: vec![
                EchoAst::Number(100),
            ],
            catch: Some(CatchClause {
                error_var: None,
                body: vec![
                    EchoAst::Number(-1),
                ],
            }),
            finally: Some(vec![
                EchoAst::String("always executed".to_string()),
            ]),
        };
        
        let result = jit.eval(&ast).unwrap();
        
        match result {
            Value::Integer(n) => assert_eq!(n, 100),
            _ => panic!("Expected integer value"),
        }
    }

    #[test]
    fn test_jit_try_empty_body() {
        let mut jit = create_test_jit();
        
        // Create a player
        let player_id = jit.create_player("test_player").unwrap();
        jit.switch_player(player_id).unwrap();
        
        // Test try block with empty body
        let ast = EchoAst::Try {
            body: vec![],
            catch: Some(CatchClause {
                error_var: Some("err".to_string()),
                body: vec![
                    EchoAst::String("caught error".to_string()),
                ],
            }),
            finally: None,
        };
        
        let result = jit.eval(&ast).unwrap();
        
        match result {
            Value::Null => {}, // Empty body returns null
            _ => panic!("Expected null value"),
        }
    }

    #[test]
    fn test_jit_try_multiple_statements() {
        let mut jit = create_test_jit();
        
        // Create a player
        let player_id = jit.create_player("test_player").unwrap();
        jit.switch_player(player_id).unwrap();
        
        // Test try block with multiple statements
        let ast = EchoAst::Try {
            body: vec![
                EchoAst::Number(10),
                EchoAst::Number(20),
                EchoAst::Add {
                    left: Box::new(EchoAst::Number(5)),
                    right: Box::new(EchoAst::Number(15)),
                },
            ],
            catch: Some(CatchClause {
                error_var: Some("error".to_string()),
                body: vec![
                    EchoAst::Number(0),
                ],
            }),
            finally: Some(vec![
                EchoAst::String("done".to_string()),
            ]),
        };
        
        let result = jit.eval(&ast).unwrap();
        
        match result {
            Value::Integer(n) => assert_eq!(n, 20), // Last statement result
            _ => panic!("Expected integer value"),
        }
    }

    #[test]
    fn test_jit_compile_try_catch() {
        let mut jit = create_test_jit();
        
        // Skip test if JIT is not enabled
        if !jit.is_jit_enabled() {
            println!("Skipping try/catch compilation test - JIT not enabled");
            return;
        }
        
        // Test that compile_ast handles try/catch expressions
        let try_expressions = vec![
            EchoAst::Try {
                body: vec![EchoAst::Number(42)],
                catch: None,
                finally: None,
            },
            EchoAst::Try {
                body: vec![EchoAst::Number(42)],
                catch: Some(CatchClause {
                    error_var: Some("e".to_string()),
                    body: vec![EchoAst::Number(0)],
                }),
                finally: None,
            },
            EchoAst::Try {
                body: vec![EchoAst::Number(42)],
                catch: None,
                finally: Some(vec![EchoAst::String("cleanup".to_string())]),
            },
        ];
        
        for expr in try_expressions {
            match jit.compile_ast(&expr) {
                Ok(()) => println!("Successfully compiled: {:?}", expr),
                Err(e) => {
                    if e.to_string().contains("falling back to interpreter") {
                        println!("Try/Catch falls back to interpreter as expected: {:?}", expr);
                    } else {
                        panic!("Failed to compile {:?}: {}", expr, e);
                    }
                }
            }
        }
    }
}