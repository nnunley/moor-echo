use super::*;
use crate::ast::{EchoAst, DestructuringTarget};
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
fn test_optional_destructuring_parameters() -> anyhow::Result<()> {
    let (mut evaluator, player_id) = create_test_evaluator();
    
    // Test 1: Optional parameter with default value used
    let destructuring_ast = EchoAst::DestructuringAssignment {
        targets: vec![
            DestructuringTarget::Simple("a".to_string()),
            DestructuringTarget::Optional { 
                name: "b".to_string(), 
                default: Box::new(EchoAst::String("default".to_string()))
            },
            DestructuringTarget::Simple("c".to_string()),
        ],
        value: Box::new(EchoAst::List {
            elements: vec![
                EchoAst::Number(1),
                EchoAst::Number(2), 
                EchoAst::Number(3),
            ],
        }),
    };
    
    let result = evaluator.eval(&destructuring_ast)?;
    assert_eq!(result, Value::List(vec![
        Value::Integer(1),
        Value::Integer(2),
        Value::Integer(3),
    ]));
    
    // Check variables were bound correctly
    {
        let env = evaluator.environments.get(&player_id).unwrap();
        assert_eq!(env.variables.get("a"), Some(&Value::Integer(1)));
        assert_eq!(env.variables.get("b"), Some(&Value::Integer(2)));
        assert_eq!(env.variables.get("c"), Some(&Value::Integer(3)));
    }
    
    // Test 2: Optional parameter with default value when list is too short
    let destructuring_ast2 = EchoAst::DestructuringAssignment {
        targets: vec![
            DestructuringTarget::Simple("x".to_string()),
            DestructuringTarget::Optional {
                name: "y".to_string(),
                default: Box::new(EchoAst::String("missing".to_string()))
            },
            DestructuringTarget::Optional {
                name: "z".to_string(),
                default: Box::new(EchoAst::Number(99))
            },
        ],
        value: Box::new(EchoAst::List {
            elements: vec![EchoAst::Number(10)],
        }),
    };
    
    evaluator.eval(&destructuring_ast2)?;
    {
        let env = evaluator.environments.get(&player_id).unwrap();
        assert_eq!(env.variables.get("x"), Some(&Value::Integer(10)));
        assert_eq!(env.variables.get("y"), Some(&Value::String("missing".to_string())));
        assert_eq!(env.variables.get("z"), Some(&Value::Integer(99)));
    }
    
    // Test 3: All optional parameters with empty list
    let destructuring_ast3 = EchoAst::DestructuringAssignment {
        targets: vec![
            DestructuringTarget::Optional {
                name: "p1".to_string(),
                default: Box::new(EchoAst::Number(1))
            },
            DestructuringTarget::Optional {
                name: "p2".to_string(), 
                default: Box::new(EchoAst::Number(2))
            },
            DestructuringTarget::Optional {
                name: "p3".to_string(),
                default: Box::new(EchoAst::Number(3))
            },
        ],
        value: Box::new(EchoAst::List { elements: vec![] }),
    };
    
    evaluator.eval(&destructuring_ast3)?;
    {
        let env = evaluator.environments.get(&player_id).unwrap();
        assert_eq!(env.variables.get("p1"), Some(&Value::Integer(1)));
        assert_eq!(env.variables.get("p2"), Some(&Value::Integer(2)));
        assert_eq!(env.variables.get("p3"), Some(&Value::Integer(3)));
    }
    
    Ok(())
}

#[test] 
fn test_optional_destructuring_edge_cases() -> anyhow::Result<()> {
    let (mut evaluator, player_id) = create_test_evaluator();
    
    println!("Starting test_optional_destructuring_edge_cases");
    
    // Test with null default
    let destructuring_ast = EchoAst::DestructuringAssignment {
        targets: vec![
            DestructuringTarget::Optional {
                name: "a".to_string(),
                default: Box::new(EchoAst::Null)
            },
            DestructuringTarget::Optional {
                name: "b".to_string(),
                default: Box::new(EchoAst::Null)
            },
        ],
        value: Box::new(EchoAst::List {
            elements: vec![EchoAst::Number(42)],
        }),
    };
    
    println!("About to evaluate first destructuring assignment");
    evaluator.eval(&destructuring_ast)?;
    println!("First destructuring assignment evaluated successfully");
    {
        let env = evaluator.environments.get(&player_id).unwrap();
        assert_eq!(env.variables.get("a"), Some(&Value::Integer(42)));
        assert_eq!(env.variables.get("b"), Some(&Value::Null));
    }
    
    // Test with string default to isolate the issue
    let destructuring_ast2 = EchoAst::DestructuringAssignment {
        targets: vec![
            DestructuringTarget::Optional {
                name: "str_default".to_string(),
                default: Box::new(EchoAst::String("test_string".to_string()))
            },
        ],
        value: Box::new(EchoAst::List { elements: vec![] }),
    };
    
    evaluator.eval(&destructuring_ast2)?;
    {
        let env = evaluator.environments.get(&player_id).unwrap();
        assert_eq!(env.variables.get("str_default"), Some(&Value::String("test_string".to_string())));
    }
    
    // Test with number as default
    let destructuring_ast3 = EchoAst::DestructuringAssignment {
        targets: vec![
            DestructuringTarget::Optional {
                name: "num_default".to_string(),
                default: Box::new(EchoAst::Number(42))
            },
        ],
        value: Box::new(EchoAst::List { elements: vec![] }),
    };
    
    evaluator.eval(&destructuring_ast3)?;
    {
        let env = evaluator.environments.get(&player_id).unwrap();
        assert_eq!(env.variables.get("num_default"), Some(&Value::Integer(42)));
    }
    
    Ok(())
}