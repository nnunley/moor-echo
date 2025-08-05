use super::*;
use crate::ast::{EchoAst, CatchClause};
use crate::storage::Storage;
use std::sync::Arc;

fn create_test_evaluator() -> (Evaluator, ObjectId) {
    let temp_dir = tempfile::tempdir().unwrap();
    let storage = Arc::new(Storage::new(temp_dir.path()).unwrap());
    let mut evaluator = Evaluator::new(storage);
    let player_id = evaluator.create_player("test_player").unwrap();
    evaluator.switch_player(player_id).unwrap();
    (evaluator, player_id)
}

#[test]
fn test_try_catch_basic() -> anyhow::Result<()> {
    let (mut evaluator, _player_id) = create_test_evaluator();
    
    // Test try/catch with division by zero
    let try_ast = EchoAst::Try {
        body: vec![
            EchoAst::Divide {
                left: Box::new(EchoAst::Number(10)),
                right: Box::new(EchoAst::Number(0)),
            },
        ],
        catch: Some(CatchClause {
            error_var: Some("e".to_string()),
            body: vec![
                EchoAst::String("caught error".to_string()),
            ],
        }),
        finally: None,
    };
    
    let result = evaluator.eval(&try_ast)?;
    assert_eq!(result, Value::String("caught error".to_string()));
    
    Ok(())
}

#[test]
fn test_try_catch_error_variable() -> anyhow::Result<()> {
    let (mut evaluator, _player_id) = create_test_evaluator();
    
    // Test that error variable gets set correctly
    let try_ast = EchoAst::Try {
        body: vec![
            EchoAst::PropertyAccess {
                object: Box::new(EchoAst::ObjectRef(999)), // Non-existent object
                property: "test".to_string(),
            },
        ],
        catch: Some(CatchClause {
            error_var: Some("error_msg".to_string()),
            body: vec![
                EchoAst::Identifier("error_msg".to_string()),
            ],
        }),
        finally: None,
    };
    
    let result = evaluator.eval(&try_ast)?;
    
    // Check that we got an error message
    match result {
        Value::String(msg) => {
            assert!(msg.contains("Object reference"));
            assert!(msg.contains("not found"));
        }
        _ => panic!("Expected string error message"),
    }
    
    Ok(())
}

#[test]
fn test_try_catch_finally() -> anyhow::Result<()> {
    let (mut evaluator, _player_id) = create_test_evaluator();
    
    // Test try/catch/finally execution order
    let try_ast = EchoAst::Try {
        body: vec![
            EchoAst::Assignment {
                target: LValue::Binding {
                    binding_type: BindingType::None,
                    pattern: BindingPattern::Identifier("x".to_string()),
                },
                value: Box::new(EchoAst::Number(1)),
            },
            EchoAst::Divide {
                left: Box::new(EchoAst::Number(10)),
                right: Box::new(EchoAst::Number(0)), // This will error
            },
        ],
        catch: Some(CatchClause {
            error_var: None,
            body: vec![
                EchoAst::Assignment {
                    target: LValue::Binding {
                        binding_type: BindingType::None,
                        pattern: BindingPattern::Identifier("x".to_string()),
                    },
                    value: Box::new(EchoAst::Number(2)),
                },
            ],
        }),
        finally: Some(vec![
            EchoAst::Assignment {
                target: LValue::Binding {
                    binding_type: BindingType::None,
                    pattern: BindingPattern::Identifier("x".to_string()),
                },
                value: Box::new(EchoAst::Number(3)),
            },
        ]),
    };
    
    evaluator.eval(&try_ast)?;
    
    // Check that x was set by finally block (which runs last)
    let x_val = evaluator.eval(&EchoAst::Identifier("x".to_string()))?;
    assert_eq!(x_val, Value::Integer(3));
    
    Ok(())
}

#[test]
fn test_try_without_catch() -> anyhow::Result<()> {
    let (mut evaluator, _player_id) = create_test_evaluator();
    
    // Test try without catch (should re-throw error)
    let try_ast = EchoAst::Try {
        body: vec![
            EchoAst::Divide {
                left: Box::new(EchoAst::Number(10)),
                right: Box::new(EchoAst::Number(0)),
            },
        ],
        catch: None,
        finally: Some(vec![
            EchoAst::String("finally executed".to_string()),
        ]),
    };
    
    // Should error even with finally
    let result = evaluator.eval(&try_ast);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Division by zero"));
    
    Ok(())
}

#[test]
fn test_try_no_error() -> anyhow::Result<()> {
    let (mut evaluator, _player_id) = create_test_evaluator();
    
    // Test try block that succeeds
    let try_ast = EchoAst::Try {
        body: vec![
            EchoAst::Add {
                left: Box::new(EchoAst::Number(2)),
                right: Box::new(EchoAst::Number(3)),
            },
        ],
        catch: Some(CatchClause {
            error_var: None,
            body: vec![
                EchoAst::String("should not execute".to_string()),
            ],
        }),
        finally: None,
    };
    
    let result = evaluator.eval(&try_ast)?;
    assert_eq!(result, Value::Integer(5));
    
    Ok(())
}

#[test]
fn test_nested_try_catch() -> anyhow::Result<()> {
    let (mut evaluator, _player_id) = create_test_evaluator();
    
    // Test nested try/catch blocks
    let inner_try = EchoAst::Try {
        body: vec![
            EchoAst::Divide {
                left: Box::new(EchoAst::Number(10)),
                right: Box::new(EchoAst::Number(0)),
            },
        ],
        catch: Some(CatchClause {
            error_var: Some("inner_error".to_string()),
            body: vec![
                EchoAst::String("inner catch".to_string()),
            ],
        }),
        finally: None,
    };
    
    let outer_try = EchoAst::Try {
        body: vec![inner_try],
        catch: Some(CatchClause {
            error_var: Some("outer_error".to_string()),
            body: vec![
                EchoAst::String("outer catch".to_string()),
            ],
        }),
        finally: None,
    };
    
    let result = evaluator.eval(&outer_try)?;
    // Inner catch should handle the error
    assert_eq!(result, Value::String("inner catch".to_string()));
    
    Ok(())
}

#[test]
fn test_error_in_finally() -> anyhow::Result<()> {
    let (mut evaluator, _player_id) = create_test_evaluator();
    
    // Test error in finally block
    let try_ast = EchoAst::Try {
        body: vec![
            EchoAst::Number(42), // Success
        ],
        catch: None,
        finally: Some(vec![
            EchoAst::Divide {
                left: Box::new(EchoAst::Number(10)),
                right: Box::new(EchoAst::Number(0)), // Error in finally
            },
        ]),
    };
    
    // Error in finally should override success result
    let result = evaluator.eval(&try_ast);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Division by zero"));
    
    Ok(())
}

#[test]
fn test_raise_function() -> anyhow::Result<()> {
    let (mut evaluator, _player_id) = create_test_evaluator();
    
    // Test raise() function throws an error
    let raise_ast = EchoAst::FunctionCall {
        name: "raise".to_string(),
        args: vec![
            EchoAst::String("Custom error message".to_string()),
        ],
    };
    
    let result = evaluator.eval(&raise_ast);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().to_string(), "Custom error message");
    
    Ok(())
}

#[test]
fn test_raise_with_try_catch() -> anyhow::Result<()> {
    let (mut evaluator, _player_id) = create_test_evaluator();
    
    // Test raise() inside try/catch
    let try_ast = EchoAst::Try {
        body: vec![
            EchoAst::FunctionCall {
                name: "raise".to_string(),
                args: vec![
                    EchoAst::String("Raised exception".to_string()),
                ],
            },
        ],
        catch: Some(CatchClause {
            error_var: Some("e".to_string()),
            body: vec![
                EchoAst::Identifier("e".to_string()),
            ],
        }),
        finally: None,
    };
    
    let result = evaluator.eval(&try_ast)?;
    assert_eq!(result, Value::String("Raised exception".to_string()));
    
    Ok(())
}

#[test]
fn test_raise_with_non_string() -> anyhow::Result<()> {
    let (mut evaluator, _player_id) = create_test_evaluator();
    
    // Test raise() with non-string argument (should convert to string)
    let try_ast = EchoAst::Try {
        body: vec![
            EchoAst::FunctionCall {
                name: "raise".to_string(),
                args: vec![
                    EchoAst::Number(42),
                ],
            },
        ],
        catch: Some(CatchClause {
            error_var: Some("err".to_string()),
            body: vec![
                EchoAst::Identifier("err".to_string()),
            ],
        }),
        finally: None,
    };
    
    let result = evaluator.eval(&try_ast)?;
    assert_eq!(result, Value::String("42".to_string()));
    
    Ok(())
}

#[test]
fn test_raise_wrong_args() -> anyhow::Result<()> {
    let (mut evaluator, _player_id) = create_test_evaluator();
    
    // Test raise() with wrong number of arguments
    let raise_ast = EchoAst::FunctionCall {
        name: "raise".to_string(),
        args: vec![
            EchoAst::String("error1".to_string()),
            EchoAst::String("error2".to_string()),
        ],
    };
    
    let result = evaluator.eval(&raise_ast);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("exactly one argument"));
    
    Ok(())
}