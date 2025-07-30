//! Tests for JIT compilation of control flow operations

#[cfg(all(test, feature = "jit"))]
mod tests {
    use super::super::*;
    use crate::ast::{EchoAst, LValue, BindingType, BindingPattern};
    use crate::storage::Storage;
    use std::sync::Arc;
    use tempfile::TempDir;

    fn create_test_jit() -> JitEvaluator {
        let temp_dir = TempDir::new().unwrap();
        let storage = Arc::new(Storage::new(temp_dir.path()).unwrap());
        JitEvaluator::new_with_fallback(storage)
    }

    #[test]
    fn test_jit_if_statement() {
        let mut jit = create_test_jit();
        
        // Create a player
        let player_id = jit.create_player("test_player").unwrap();
        jit.switch_player(player_id).unwrap();
        
        // Test if statement with true condition
        let ast = EchoAst::If {
            condition: Box::new(EchoAst::Boolean(true)),
            then_branch: vec![EchoAst::Number(42)],
            else_branch: Some(vec![EchoAst::Number(24)]),
        };
        let result = jit.eval(&ast).unwrap();
        
        match result {
            Value::Integer(n) => assert_eq!(n, 42),
            _ => panic!("Expected integer value"),
        }
        
        // Test if statement with false condition
        let ast = EchoAst::If {
            condition: Box::new(EchoAst::Boolean(false)),
            then_branch: vec![EchoAst::Number(42)],
            else_branch: Some(vec![EchoAst::Number(24)]),
        };
        let result = jit.eval(&ast).unwrap();
        
        match result {
            Value::Integer(n) => assert_eq!(n, 24),
            _ => panic!("Expected integer value"),
        }
        
        // Test if statement without else
        let ast = EchoAst::If {
            condition: Box::new(EchoAst::Boolean(false)),
            then_branch: vec![EchoAst::Number(42)],
            else_branch: None,
        };
        let result = jit.eval(&ast).unwrap();
        
        match result {
            Value::Null => {},
            _ => panic!("Expected null value"),
        }
    }

    #[test]
    fn test_jit_if_with_comparison() {
        let mut jit = create_test_jit();
        
        // Create a player
        let player_id = jit.create_player("test_player").unwrap();
        jit.switch_player(player_id).unwrap();
        
        // Test if statement with comparison condition
        let ast = EchoAst::If {
            condition: Box::new(EchoAst::GreaterThan {
                left: Box::new(EchoAst::Number(10)),
                right: Box::new(EchoAst::Number(5)),
            }),
            then_branch: vec![EchoAst::Add {
                left: Box::new(EchoAst::Number(100)),
                right: Box::new(EchoAst::Number(23)),
            }],
            else_branch: Some(vec![EchoAst::Number(999)]),
        };
        let result = jit.eval(&ast).unwrap();
        
        match result {
            Value::Integer(n) => assert_eq!(n, 123),
            _ => panic!("Expected integer value"),
        }
    }

    #[test]
    fn test_jit_nested_if() {
        let mut jit = create_test_jit();
        
        // Create a player
        let player_id = jit.create_player("test_player").unwrap();
        jit.switch_player(player_id).unwrap();
        
        // Test nested if statements
        let ast = EchoAst::If {
            condition: Box::new(EchoAst::Boolean(true)),
            then_branch: vec![EchoAst::If {
                condition: Box::new(EchoAst::LessThan {
                    left: Box::new(EchoAst::Number(5)),
                    right: Box::new(EchoAst::Number(10)),
                }),
                then_branch: vec![EchoAst::Number(111)],
                else_branch: Some(vec![EchoAst::Number(222)]),
            }],
            else_branch: Some(vec![EchoAst::Number(333)]),
        };
        let result = jit.eval(&ast).unwrap();
        
        match result {
            Value::Integer(n) => assert_eq!(n, 111),
            _ => panic!("Expected integer value"),
        }
    }

    #[test]
    fn test_jit_compile_if() {
        let mut jit = create_test_jit();
        
        // Skip test if JIT is not enabled
        if !jit.is_jit_enabled() {
            println!("Skipping if compilation test - JIT not enabled");
            return;
        }
        
        // Test that compile_ast handles if statements
        let operations = vec![
            EchoAst::If {
                condition: Box::new(EchoAst::Boolean(true)),
                then_branch: vec![EchoAst::Number(42)],
                else_branch: Some(vec![EchoAst::Number(24)]),
            },
            EchoAst::If {
                condition: Box::new(EchoAst::GreaterThan {
                    left: Box::new(EchoAst::Number(10)),
                    right: Box::new(EchoAst::Number(5)),
                }),
                then_branch: vec![EchoAst::Number(100)],
                else_branch: None,
            },
        ];
        
        for op in operations {
            match jit.compile_ast(&op) {
                Ok(()) => println!("Successfully compiled: {:?}", op),
                Err(e) => {
                    // If statements require control flow support
                    if e.to_string().contains("falling back to interpreter") {
                        println!("If statement falls back to interpreter as expected: {:?}", op);
                    } else {
                        panic!("Failed to compile {:?}: {}", op, e);
                    }
                }
            }
        }
    }
    
    #[test]
    fn test_jit_while_loop() {
        let mut jit = create_test_jit();
        
        // Create a player
        let player_id = jit.create_player("test_player").unwrap();
        jit.switch_player(player_id).unwrap();
        
        // First set up a counter variable
        let init_ast = EchoAst::Assignment {
            target: LValue::Binding {
                binding_type: BindingType::Let,
                pattern: BindingPattern::Identifier("counter".to_string()),
            },
            value: Box::new(EchoAst::Number(0)),
        };
        jit.eval(&init_ast).unwrap();
        
        // Test while loop that increments counter to 5
        let ast = EchoAst::While {
            label: None,
            condition: Box::new(EchoAst::LessThan {
                left: Box::new(EchoAst::Identifier("counter".to_string())),
                right: Box::new(EchoAst::Number(5)),
            }),
            body: vec![EchoAst::Assignment {
                target: LValue::Binding {
                    binding_type: BindingType::Let,
                    pattern: BindingPattern::Identifier("counter".to_string()),
                },
                value: Box::new(EchoAst::Add {
                    left: Box::new(EchoAst::Identifier("counter".to_string())),
                    right: Box::new(EchoAst::Number(1)),
                }),
            }],
        };
        let result = jit.eval(&ast).unwrap();
        
        // While loops return null
        match result {
            Value::Null => {},
            _ => panic!("Expected null value"),
        }
        
        // Check that counter was incremented to 5
        let check_ast = EchoAst::Identifier("counter".to_string());
        let counter_result = jit.eval(&check_ast).unwrap();
        
        match counter_result {
            Value::Integer(n) => assert_eq!(n, 5),
            _ => panic!("Expected integer value"),
        }
    }

    #[test]
    fn test_jit_for_loop() {
        let mut jit = create_test_jit();
        
        // Create a player
        let player_id = jit.create_player("test_player").unwrap();
        jit.switch_player(player_id).unwrap();
        
        // Set up a sum variable
        let init_ast = EchoAst::Assignment {
            target: LValue::Binding {
                binding_type: BindingType::Let,
                pattern: BindingPattern::Identifier("sum".to_string()),
            },
            value: Box::new(EchoAst::Number(0)),
        };
        jit.eval(&init_ast).unwrap();
        
        // Test for loop over a list
        let ast = EchoAst::For {
            label: None,
            variable: "x".to_string(),
            collection: Box::new(EchoAst::List {
                elements: vec![
                    EchoAst::Number(1),
                    EchoAst::Number(2),
                    EchoAst::Number(3),
                    EchoAst::Number(4),
                    EchoAst::Number(5),
                ],
            }),
            body: vec![EchoAst::Assignment {
                target: LValue::Binding {
                    binding_type: BindingType::Let,
                    pattern: BindingPattern::Identifier("sum".to_string()),
                },
                value: Box::new(EchoAst::Add {
                    left: Box::new(EchoAst::Identifier("sum".to_string())),
                    right: Box::new(EchoAst::Identifier("x".to_string())),
                }),
            }],
        };
        let result = jit.eval(&ast).unwrap();
        
        // For loops return null
        match result {
            Value::Null => {},
            _ => panic!("Expected null value"),
        }
        
        // Check that sum is 15 (1+2+3+4+5)
        let check_ast = EchoAst::Identifier("sum".to_string());
        let sum_result = jit.eval(&check_ast).unwrap();
        
        match sum_result {
            Value::Integer(n) => assert_eq!(n, 15),
            _ => panic!("Expected integer value"),
        }
    }

    #[test]
    fn test_jit_return() {
        let mut jit = create_test_jit();
        
        // Create a player
        let player_id = jit.create_player("test_player").unwrap();
        jit.switch_player(player_id).unwrap();
        
        // Test return statement
        let ast = EchoAst::Return {
            value: Some(Box::new(EchoAst::Number(42))),
        };
        let result = jit.eval(&ast).unwrap();
        
        match result {
            Value::Integer(n) => assert_eq!(n, 42),
            _ => panic!("Expected integer value"),
        }
        
        // Test return without value
        let ast = EchoAst::Return { value: None };
        let result = jit.eval(&ast).unwrap();
        
        match result {
            Value::Null => {},
            _ => panic!("Expected null value"),
        }
    }

    #[test]
    fn test_jit_break_continue() {
        let mut jit = create_test_jit();
        
        // Create a player
        let player_id = jit.create_player("test_player").unwrap();
        jit.switch_player(player_id).unwrap();
        
        // Set up counter
        let init_ast = EchoAst::Assignment {
            target: LValue::Binding {
                binding_type: BindingType::Let,
                pattern: BindingPattern::Identifier("counter".to_string()),
            },
            value: Box::new(EchoAst::Number(0)),
        };
        jit.eval(&init_ast).unwrap();
        
        // Test while loop with break
        let ast = EchoAst::While {
            label: None,
            condition: Box::new(EchoAst::Boolean(true)),
            body: vec![
                EchoAst::Assignment {
                    target: LValue::Binding {
                        binding_type: BindingType::Let,
                        pattern: BindingPattern::Identifier("counter".to_string()),
                    },
                    value: Box::new(EchoAst::Add {
                        left: Box::new(EchoAst::Identifier("counter".to_string())),
                        right: Box::new(EchoAst::Number(1)),
                    }),
                },
                EchoAst::If {
                    condition: Box::new(EchoAst::GreaterEqual {
                        left: Box::new(EchoAst::Identifier("counter".to_string())),
                        right: Box::new(EchoAst::Number(5)),
                    }),
                    then_branch: vec![EchoAst::Break { label: None }],
                    else_branch: None,
                },
            ],
        };
        jit.eval(&ast).unwrap();
        
        // Check counter is 5
        let check_ast = EchoAst::Identifier("counter".to_string());
        let result = jit.eval(&check_ast).unwrap();
        
        match result {
            Value::Integer(n) => assert_eq!(n, 5),
            _ => panic!("Expected integer value"),
        }
    }

    #[test]
    fn test_jit_compile_loops() {
        let mut jit = create_test_jit();
        
        // Skip test if JIT is not enabled
        if !jit.is_jit_enabled() {
            println!("Skipping loop compilation test - JIT not enabled");
            return;
        }
        
        // Test that compile_ast handles loops
        let operations = vec![
            EchoAst::While {
                label: None,
                condition: Box::new(EchoAst::Boolean(false)),
                body: vec![EchoAst::Number(42)],
            },
            EchoAst::For {
                label: None,
                variable: "i".to_string(),
                collection: Box::new(EchoAst::List {
                    elements: vec![EchoAst::Number(1), EchoAst::Number(2)],
                }),
                body: vec![EchoAst::Number(0)],
            },
            EchoAst::Return {
                value: Some(Box::new(EchoAst::Number(99))),
            },
            EchoAst::Break { label: None },
            EchoAst::Continue { label: None },
        ];
        
        for op in operations {
            match jit.compile_ast(&op) {
                Ok(()) => println!("Successfully compiled: {:?}", op),
                Err(e) => {
                    // Loops require control flow support
                    if e.to_string().contains("falling back to interpreter") {
                        println!("Loop falls back to interpreter as expected: {:?}", op);
                    } else {
                        panic!("Failed to compile {:?}: {}", op, e);
                    }
                }
            }
        }
    }
}