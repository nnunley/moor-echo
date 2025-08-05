use super::*;
use crate::ast::{EchoAst, ObjectMember};
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
fn test_verb_multiple_names() -> anyhow::Result<()> {
    let (mut evaluator, _player_id) = create_test_evaluator();
    
    // Create an object with a verb that has multiple names using AST
    let obj_def = EchoAst::ObjectDef {
        name: "TestObject".to_string(),
        parent: Some("$root".to_string()),
        members: vec![
            ObjectMember::Verb {
                name: "l look examine".to_string(),
                args: vec![],
                body: vec![
                    EchoAst::Return {
                        value: Some(Box::new(EchoAst::String("You look around.".to_string()))),
                    }
                ],
                permissions: None,
                required_capabilities: vec![],
            },
        ],
    };
    
    // Evaluate the object definition
    let obj_val = evaluator.eval(&obj_def)?;
    let _obj_id = match obj_val {
        Value::Object(id) => id,
        _ => panic!("Expected object value"),
    };
    
    // Test calling with different names
    let test_cases = vec!["l", "look", "examine"];
    
    for verb_name in test_cases {
        let method_call = EchoAst::MethodCall {
            object: Box::new(EchoAst::Identifier("TestObject".to_string())),
            method: verb_name.to_string(),
            args: vec![],
        };
        
        let result = evaluator.eval(&method_call)?;
        assert_eq!(result, Value::String("You look around.".to_string()),
            "Failed for verb name: {}", verb_name);
    }
    
    Ok(())
}

#[test]
fn test_verb_star_pattern() -> anyhow::Result<()> {
    let (mut evaluator, _player_id) = create_test_evaluator();
    
    // Create an object with pattern verbs
    let obj_def = EchoAst::ObjectDef {
        name: "PatternObject".to_string(),
        parent: Some("$root".to_string()),
        members: vec![
            ObjectMember::Verb {
                name: "foo*bar".to_string(),
                args: vec![],
                body: vec![
                    EchoAst::Return {
                        value: Some(Box::new(EchoAst::String("Foo pattern matched!".to_string()))),
                    }
                ],
                permissions: None,
                required_capabilities: vec![],
            },
            ObjectMember::Verb {
                name: "*".to_string(),
                args: vec![],
                body: vec![
                    EchoAst::Return {
                        value: Some(Box::new(EchoAst::String("Wildcard matched!".to_string()))),
                    }
                ],
                permissions: None,
                required_capabilities: vec![],
            },
        ],
    };
    
    // Evaluate the object definition
    evaluator.eval(&obj_def)?;
    
    // Test foo*bar pattern matches
    let foo_tests = vec![
        ("foo", "Foo pattern matched!"),
        ("foob", "Foo pattern matched!"),
        ("fooba", "Foo pattern matched!"),
        ("foobar", "Foo pattern matched!"),
    ];
    
    for (verb_name, expected) in foo_tests {
        let method_call = EchoAst::MethodCall {
            object: Box::new(EchoAst::Identifier("PatternObject".to_string())),
            method: verb_name.to_string(),
            args: vec![],
        };
        
        let result = evaluator.eval(&method_call)?;
        assert_eq!(result, Value::String(expected.to_string()),
            "Failed for verb name: {}", verb_name);
    }
    
    // Test wildcard pattern matches anything else
    let wildcard_tests = vec!["unknown", "random", "anything"];
    
    for verb_name in wildcard_tests {
        let method_call = EchoAst::MethodCall {
            object: Box::new(EchoAst::Identifier("PatternObject".to_string())),
            method: verb_name.to_string(),
            args: vec![],
        };
        
        let result = evaluator.eval(&method_call)?;
        assert_eq!(result, Value::String("Wildcard matched!".to_string()),
            "Failed for verb name: {}", verb_name);
    }
    
    Ok(())
}

#[test]
fn test_pronoun_pattern() -> anyhow::Result<()> {
    let (mut evaluator, _player_id) = create_test_evaluator();
    
    // Create an object with pronoun_* pattern
    let obj_def = EchoAst::ObjectDef {
        name: "PronounObject".to_string(),
        parent: Some("$root".to_string()),
        members: vec![
            ObjectMember::Verb {
                name: "pronoun_*".to_string(),
                args: vec![],
                body: vec![
                    EchoAst::Return {
                        value: Some(Box::new(EchoAst::String("Pronoun method called!".to_string()))),
                    }
                ],
                permissions: None,
                required_capabilities: vec![],
            },
        ],
    };
    
    // Evaluate the object definition
    evaluator.eval(&obj_def)?;
    
    // Test pronoun_* pattern matches
    let pronoun_tests = vec!["pronoun_", "pronoun_sub", "pronoun_obj", "pronoun_possessive"];
    
    for verb_name in pronoun_tests {
        let method_call = EchoAst::MethodCall {
            object: Box::new(EchoAst::Identifier("PronounObject".to_string())),
            method: verb_name.to_string(),
            args: vec![],
        };
        
        let result = evaluator.eval(&method_call)?;
        assert_eq!(result, Value::String("Pronoun method called!".to_string()),
            "Failed for verb name: {}", verb_name);
    }
    
    // Test that "pronoun" without underscore doesn't match
    let method_call = EchoAst::MethodCall {
        object: Box::new(EchoAst::Identifier("PronounObject".to_string())),
        method: "pronoun".to_string(),
        args: vec![],
    };
    
    let result = evaluator.eval(&method_call);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not found"));
    
    Ok(())
}

#[test]
fn test_verb_pattern_priority() -> anyhow::Result<()> {
    let (mut evaluator, _player_id) = create_test_evaluator();
    
    // Create an object with overlapping patterns
    let obj_def = EchoAst::ObjectDef {
        name: "PriorityObject".to_string(),
        parent: Some("$root".to_string()),
        members: vec![
            ObjectMember::Verb {
                name: "test".to_string(),
                args: vec![],
                body: vec![
                    EchoAst::Return {
                        value: Some(Box::new(EchoAst::String("Exact match!".to_string()))),
                    }
                ],
                permissions: None,
                required_capabilities: vec![],
            },
            ObjectMember::Verb {
                name: "te*".to_string(),
                args: vec![],
                body: vec![
                    EchoAst::Return {
                        value: Some(Box::new(EchoAst::String("Pattern match!".to_string()))),
                    }
                ],
                permissions: None,
                required_capabilities: vec![],
            },
            ObjectMember::Verb {
                name: "*".to_string(),
                args: vec![],
                body: vec![
                    EchoAst::Return {
                        value: Some(Box::new(EchoAst::String("Wildcard match!".to_string()))),
                    }
                ],
                permissions: None,
                required_capabilities: vec![],
            },
        ],
    };
    
    // Evaluate the object definition
    evaluator.eval(&obj_def)?;
    
    // Test that exact match has highest priority
    let method_call = EchoAst::MethodCall {
        object: Box::new(EchoAst::Identifier("PriorityObject".to_string())),
        method: "test".to_string(),
        args: vec![],
    };
    
    let result = evaluator.eval(&method_call)?;
    assert_eq!(result, Value::String("Exact match!".to_string()));
    
    // Test that pattern match is used when no exact match
    let method_call = EchoAst::MethodCall {
        object: Box::new(EchoAst::Identifier("PriorityObject".to_string())),
        method: "testing".to_string(),
        args: vec![],
    };
    
    let result = evaluator.eval(&method_call)?;
    assert_eq!(result, Value::String("Pattern match!".to_string()));
    
    // Test that wildcard is used as last resort
    let method_call = EchoAst::MethodCall {
        object: Box::new(EchoAst::Identifier("PriorityObject".to_string())),
        method: "unknown".to_string(),
        args: vec![],
    };
    
    let result = evaluator.eval(&method_call)?;
    assert_eq!(result, Value::String("Wildcard match!".to_string()));
    
    Ok(())
}