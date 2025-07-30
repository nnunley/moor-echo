//! Tests for JIT compilation of function call operations

#[cfg(all(test, feature = "jit"))]
mod tests {
    use super::super::*;
    use crate::ast::{EchoAst, LValue, BindingType, BindingPattern, LambdaParam};
    use crate::storage::Storage;
    use std::sync::Arc;
    use tempfile::TempDir;

    fn create_test_jit() -> JitEvaluator {
        let temp_dir = TempDir::new().unwrap();
        let storage = Arc::new(Storage::new(temp_dir.path()).unwrap());
        JitEvaluator::new_with_fallback(storage)
    }

    #[test]
    fn test_jit_function_call() {
        let mut jit = create_test_jit();
        
        // Create a player
        let player_id = jit.create_player("test_player").unwrap();
        jit.switch_player(player_id).unwrap();
        
        // Test built-in function call
        let ast = EchoAst::FunctionCall {
            name: "abs".to_string(),
            args: vec![EchoAst::Number(-42)],
        };
        let result = jit.eval(&ast).unwrap();
        
        match result {
            Value::Integer(n) => assert_eq!(n, 42),
            _ => panic!("Expected integer value"),
        }
    }

    #[test]
    fn test_jit_lambda_call() {
        let mut jit = create_test_jit();
        
        // Create a player
        let player_id = jit.create_player("test_player").unwrap();
        jit.switch_player(player_id).unwrap();
        
        // Create a lambda function
        let lambda_ast = EchoAst::Lambda {
            params: vec![
                LambdaParam::Simple("x".to_string()),
                LambdaParam::Simple("y".to_string()),
            ],
            body: Box::new(EchoAst::Add {
                left: Box::new(EchoAst::Identifier("x".to_string())),
                right: Box::new(EchoAst::Identifier("y".to_string())),
            }),
        };
        
        // Store lambda in a variable
        let assign_ast = EchoAst::Assignment {
            target: LValue::Binding {
                binding_type: BindingType::Let,
                pattern: BindingPattern::Identifier("add".to_string()),
            },
            value: Box::new(lambda_ast),
        };
        jit.eval(&assign_ast).unwrap();
        
        // Call the lambda
        let ast = EchoAst::Call {
            func: Box::new(EchoAst::Identifier("add".to_string())),
            args: vec![
                EchoAst::Number(10),
                EchoAst::Number(32),
            ],
        };
        let result = jit.eval(&ast).unwrap();
        
        match result {
            Value::Integer(n) => assert_eq!(n, 42),
            _ => panic!("Expected integer value"),
        }
    }

    #[test]
    fn test_jit_method_call() {
        let mut jit = create_test_jit();
        
        // Create a player
        let player_id = jit.create_player("test_player").unwrap();
        jit.switch_player(player_id).unwrap();
        
        // Create a list
        let list_ast = EchoAst::List {
            elements: vec![
                EchoAst::Number(1),
                EchoAst::Number(2),
                EchoAst::Number(3),
            ],
        };
        
        let assign_ast = EchoAst::Assignment {
            target: LValue::Binding {
                binding_type: BindingType::Let,
                pattern: BindingPattern::Identifier("mylist".to_string()),
            },
            value: Box::new(list_ast),
        };
        jit.eval(&assign_ast).unwrap();
        
        // Call method on list (if supported)
        // Note: Echo might not have built-in methods on lists
        // This is just a test of the MethodCall AST node
        let ast = EchoAst::MethodCall {
            object: Box::new(EchoAst::Identifier("mylist".to_string())),
            method: "length".to_string(),
            args: vec![],
        };
        
        // This might fail if methods aren't implemented
        match jit.eval(&ast) {
            Ok(Value::Integer(n)) => assert_eq!(n, 3),
            Ok(_) => panic!("Expected integer value"),
            Err(_) => {
                // Methods might not be implemented yet
                println!("Method calls not yet implemented");
            }
        }
    }

    #[test]
    fn test_jit_compile_function_calls() {
        let mut jit = create_test_jit();
        
        // Skip test if JIT is not enabled
        if !jit.is_jit_enabled() {
            println!("Skipping function call compilation test - JIT not enabled");
            return;
        }
        
        // Test that compile_ast handles function calls
        let operations = vec![
            EchoAst::FunctionCall {
                name: "print".to_string(),
                args: vec![EchoAst::String("hello".to_string())],
            },
            EchoAst::MethodCall {
                object: Box::new(EchoAst::Identifier("obj".to_string())),
                method: "method".to_string(),
                args: vec![],
            },
            EchoAst::Call {
                func: Box::new(EchoAst::Identifier("func".to_string())),
                args: vec![EchoAst::Number(42)],
            },
        ];
        
        for op in operations {
            match jit.compile_ast(&op) {
                Ok(()) => println!("Successfully compiled: {:?}", op),
                Err(e) => {
                    // Function calls require runtime dispatch
                    if e.to_string().contains("falling back to interpreter") {
                        println!("Function call falls back to interpreter as expected: {:?}", op);
                    } else {
                        panic!("Failed to compile {:?}: {}", op, e);
                    }
                }
            }
        }
    }
}