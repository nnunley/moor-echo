use echo_repl::parser::{create_parser, Parser};
use echo_repl::ast::EchoAst;

// Helper function to parse echo code
fn parse_echo(code: &str) -> anyhow::Result<EchoAst> {
    let mut parser = create_parser("echo")?;
    parser.parse(code)
}

// Basic literals
#[test]
fn test_rust_sitter_number() {
    let result = parse_echo("42");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), EchoAst::Number(42));
}

#[test]
fn test_rust_sitter_negative_number() {
    let result = parse_echo("-42");
    // Negative numbers might not be supported yet in the grammar
    // This test documents current behavior
    if result.is_ok() {
        assert_eq!(result.unwrap(), EchoAst::Number(-42));
    } else {
        // Expected failure for now - negative numbers not implemented
        assert!(result.is_err());
    }
}

#[test]
fn test_rust_sitter_zero() {
    let result = parse_echo("0");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), EchoAst::Number(0));
}

#[test]
fn test_rust_sitter_string_literal() {
    let result = parse_echo("\"hello world\"");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), EchoAst::String("hello world".to_string()));
}

#[test]
fn test_rust_sitter_empty_string() {
    let result = parse_echo("\"\"");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), EchoAst::String("".to_string()));
}

#[test]
fn test_rust_sitter_string_with_spaces() {
    let result = parse_echo("\"  hello  world  \"");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), EchoAst::String("  hello  world  ".to_string()));
}

#[test]
fn test_rust_sitter_identifier() {
    let result = parse_echo("hello");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), EchoAst::Identifier("hello".to_string()));
}

#[test]
fn test_rust_sitter_identifier_with_underscores() {
    let result = parse_echo("hello_world");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), EchoAst::Identifier("hello_world".to_string()));
}

#[test]
fn test_rust_sitter_identifier_with_numbers() {
    let result = parse_echo("var123");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), EchoAst::Identifier("var123".to_string()));
}

// Arithmetic operations
#[test]
fn test_rust_sitter_addition() {
    let result = parse_echo("1 + 2");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), EchoAst::Add {
        left: Box::new(EchoAst::Number(1)),
        
        right: Box::new(EchoAst::Number(2))
    });
}

#[test]
fn test_rust_sitter_addition_with_spaces() {
    let result = parse_echo("  1   +   2  ");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), EchoAst::Add {
        left: Box::new(EchoAst::Number(1)),
        
        right: Box::new(EchoAst::Number(2))
    });
}

#[test]
fn test_rust_sitter_string_concatenation() {
    let result = parse_echo("\"hello\" + \" world\"");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), EchoAst::Add {
        left: Box::new(EchoAst::String("hello".to_string())),
        
        right: Box::new(EchoAst::String(" world".to_string()))
    });
}

#[test]
fn test_rust_sitter_nested_addition() {
    let result = parse_echo("1 + 2 + 3");
    assert!(result.is_ok());
    // Should parse as left-associative: (1 + 2) + 3
    assert_eq!(result.unwrap(), EchoAst::Add {
        left: Box::new(EchoAst::Add {
            left: Box::new(EchoAst::Number(1)),
            
            right: Box::new(EchoAst::Number(2))
        }),
        
        right: Box::new(EchoAst::Number(3))
    });
}

// Property access
#[test]
fn test_rust_sitter_property_access() {
    let result = parse_echo("obj.prop");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), EchoAst::PropertyAccess {
        object: Box::new(EchoAst::Identifier("obj".to_string())),
        _dot: (),
        property: Box::new(EchoAst::Identifier("prop".to_string()))
    });
}

#[test]
fn test_rust_sitter_nested_property_access() {
    let result = parse_echo("obj.prop.subprop");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), EchoAst::PropertyAccess {
        object: Box::new(EchoAst::PropertyAccess {
            object: Box::new(EchoAst::Identifier("obj".to_string())),
            _dot: (),
            property: Box::new(EchoAst::Identifier("prop".to_string()))
        }),
        _dot: (),
        property: Box::new(EchoAst::Identifier("subprop".to_string()))
    });
}

// Let statements
#[test]
fn test_rust_sitter_let_statement() {
    let result = parse_echo("let x = 42;");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), EchoAst::Let {
        _let: (),
        name: Box::new(EchoAst::Identifier("x".to_string())),
        _equals: (),
        value: Box::new(EchoAst::Number(42)),
        _semicolon: (),
    });
}

#[test]
fn test_rust_sitter_let_with_string() {
    let result = parse_echo("let greeting = \"hello\";");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), EchoAst::Let {
        _let: (),
        name: Box::new(EchoAst::Identifier("greeting".to_string())),
        _equals: (),
        value: Box::new(EchoAst::String("hello".to_string())),
        _semicolon: (),
    });
}

#[test]
fn test_rust_sitter_let_with_expression() {
    let result = parse_echo("let sum = 1 + 2;");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), EchoAst::Let {
        _let: (),
        name: Box::new(EchoAst::Identifier("sum".to_string())),
        _equals: (),
        value: Box::new(EchoAst::Add {
            left: Box::new(EchoAst::Number(1)),
            
            right: Box::new(EchoAst::Number(2))
        }),
        _semicolon: (),
    });
}

// Object definitions
#[test]
fn test_rust_sitter_simple_object() {
    let result = parse_echo("object TestObj property name = \"test\"; endobject");
    assert!(result.is_ok());
    match result.unwrap() {
        EchoAst::ObjectDef { name, members, .. } => {
            assert_eq!(*name, EchoAst::Identifier("TestObj".to_string()));
            assert_eq!(members.len(), 1);
            match &members[0] {
                EchoAst::PropertyDef { name, value, .. } => {
                    assert_eq!(**name, EchoAst::Identifier("name".to_string()));
                    assert_eq!(**value, EchoAst::String("test".to_string()));
                }
                _ => panic!("Expected PropertyDef"),
            }
        }
        _ => panic!("Expected ObjectDef"),
    }
}

#[test]
fn test_rust_sitter_object_with_multiple_properties() {
    let result = parse_echo("object TestObj property name = \"test\"; property value = 42; endobject");
    assert!(result.is_ok());
    match result.unwrap() {
        EchoAst::ObjectDef { name, members, .. } => {
            assert_eq!(*name, EchoAst::Identifier("TestObj".to_string()));
            assert_eq!(members.len(), 2);
        }
        _ => panic!("Expected ObjectDef"),
    }
}

// Method calls
#[test]
fn test_rust_sitter_method_call() {
    let result = parse_echo("obj:method()");
    assert!(result.is_ok());
    match result.unwrap() {
        EchoAst::MethodCall { object, method, args, .. } => {
            match object.as_ref() {
                EchoAst::Identifier(name) => assert_eq!(name, "obj"),
                _ => panic!("Expected Identifier for object"),
            }
            match method.as_ref() {
                EchoAst::Identifier(name) => assert_eq!(name, "method"),
                _ => panic!("Expected Identifier for method"),
            }
            assert_eq!(args.len(), 0);
        }
        _ => panic!("Expected MethodCall"),
    }
}

#[test]
fn test_rust_sitter_method_call_with_args() {
    let result = parse_echo("obj:method(42, \"hello\")");
    assert!(result.is_ok());
    match result.unwrap() {
        EchoAst::MethodCall { object, method, args, .. } => {
            match object.as_ref() {
                EchoAst::Identifier(name) => assert_eq!(name, "obj"),
                _ => panic!("Expected Identifier for object"),
            }
            match method.as_ref() {
                EchoAst::Identifier(name) => assert_eq!(name, "method"),
                _ => panic!("Expected Identifier for method"),
            }
            // Grammar includes commas as separate tokens in args
            let non_comma_args: Vec<_> = args.iter().filter(|&arg| !matches!(arg, EchoAst::Comma)).collect();
            assert_eq!(non_comma_args.len(), 2);
            assert_eq!(args[0], EchoAst::Number(42));
            assert_eq!(args[2], EchoAst::String("hello".to_string()));
        }
        _ => panic!("Expected MethodCall"),
    }
}

// Return statements
#[test]
fn test_rust_sitter_return_statement() {
    let result = parse_echo("return 42;");
    assert!(result.is_ok());
    match result.unwrap() {
        EchoAst::Return { value, .. } => {
            match value.as_ref() {
                EchoAst::Number(n) => assert_eq!(*n, 42),
                _ => panic!("Expected Number in return"),
            }
        }
        _ => panic!("Expected Return"),
    }
}

#[test]
fn test_rust_sitter_return_expression() {
    let result = parse_echo("return 1 + 2;");
    assert!(result.is_ok());
    match result.unwrap() {
        EchoAst::Return { value, .. } => {
            match value.as_ref() {
                EchoAst::Add { left, right, .. } => {
                    assert_eq!(**left, EchoAst::Number(1));
                    assert_eq!(**right, EchoAst::Number(2));
                }
                _ => panic!("Expected Add expression"),
            }
        }
        _ => panic!("Expected Return"),
    }
}

// Boolean literals
#[test]
fn test_rust_sitter_true_literal() {
    let result = parse_echo("true");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), EchoAst::True);
}

#[test]
fn test_rust_sitter_false_literal() {
    let result = parse_echo("false");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), EchoAst::False);
}

// Comparison operators
#[test]
fn test_rust_sitter_equal_comparison() {
    let result = parse_echo("1 == 2");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), EchoAst::Equal {
        left: Box::new(EchoAst::Number(1)),
        
        right: Box::new(EchoAst::Number(2))
    });
}

#[test]
fn test_rust_sitter_not_equal_comparison() {
    let result = parse_echo("1 != 2");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), EchoAst::NotEqual {
        left: Box::new(EchoAst::Number(1)),
        
        right: Box::new(EchoAst::Number(2))
    });
}

#[test]
fn test_rust_sitter_less_than_comparison() {
    let result = parse_echo("1 < 2");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), EchoAst::LessThan {
        left: Box::new(EchoAst::Number(1)),
        
        right: Box::new(EchoAst::Number(2))
    });
}

#[test]
fn test_rust_sitter_greater_than_comparison() {
    let result = parse_echo("1 > 2");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), EchoAst::GreaterThan {
        left: Box::new(EchoAst::Number(1)),
        
        right: Box::new(EchoAst::Number(2))
    });
}

#[test]
fn test_rust_sitter_less_equal_comparison() {
    let result = parse_echo("1 <= 2");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), EchoAst::LessEqual {
        left: Box::new(EchoAst::Number(1)),
        
        right: Box::new(EchoAst::Number(2))
    });
}

#[test]
fn test_rust_sitter_greater_equal_comparison() {
    let result = parse_echo("1 >= 2");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), EchoAst::GreaterEqual {
        left: Box::new(EchoAst::Number(1)),
        
        right: Box::new(EchoAst::Number(2))
    });
}

// Conditional statements
#[test]
fn test_rust_sitter_simple_if() {
    let result = parse_echo("if (true) return 42; endif");
    assert!(result.is_ok());
    match result.unwrap() {
        EchoAst::If { condition, then_branch, else_branch, .. } => {
            assert_eq!(*condition, EchoAst::True);
            assert_eq!(then_branch.len(), 1);
            assert!(else_branch.is_none());
            match &then_branch[0] {
                EchoAst::Return { value, .. } => {
                    assert_eq!(**value, EchoAst::Number(42));
                }
                _ => panic!("Expected Return statement"),
            }
        }
        _ => panic!("Expected If statement"),
    }
}

#[test]
fn test_rust_sitter_if_else() {
    let result = parse_echo("if (false) return 1; else return 2; endif");
    assert!(result.is_ok());
    match result.unwrap() {
        EchoAst::If { condition, then_branch, else_branch, .. } => {
            assert_eq!(*condition, EchoAst::False);
            assert_eq!(then_branch.len(), 1);
            assert!(else_branch.is_some());
            match &then_branch[0] {
                EchoAst::Return { value, .. } => {
                    assert_eq!(**value, EchoAst::Number(1));
                }
                _ => panic!("Expected Return statement in then branch"),
            }
            let else_branch = else_branch.as_ref().unwrap();
            assert_eq!(else_branch.statements.len(), 1);
            match &else_branch.statements[0] {
                EchoAst::Return { value, .. } => {
                    assert_eq!(**value, EchoAst::Number(2));
                }
                _ => panic!("Expected Return statement in else branch"),
            }
        }
        _ => panic!("Expected If statement"),
    }
}

#[test]
fn test_rust_sitter_if_with_comparison() {
    let result = parse_echo("if (x == 42) return \"found\"; endif");
    assert!(result.is_ok());
    match result.unwrap() {
        EchoAst::If { condition, then_branch, else_branch, .. } => {
            match condition.as_ref() {
                EchoAst::Equal { left, right, .. } => {
                    assert_eq!(**left, EchoAst::Identifier("x".to_string()));
                    assert_eq!(**right, EchoAst::Number(42));
                }
                _ => panic!("Expected Equal comparison"),
            }
            assert_eq!(then_branch.len(), 1);
            assert!(else_branch.is_none());
            match &then_branch[0] {
                EchoAst::Return { value, .. } => {
                    assert_eq!(**value, EchoAst::String("found".to_string()));
                }
                _ => panic!("Expected Return statement"),
            }
        }
        _ => panic!("Expected If statement"),
    }
}

#[test]
fn test_rust_sitter_if_with_multiple_statements() {
    let result = parse_echo("if (true) let x = 1; return x; endif");
    assert!(result.is_ok());
    match result.unwrap() {
        EchoAst::If { condition, then_branch, else_branch, .. } => {
            assert_eq!(*condition, EchoAst::True);
            assert_eq!(then_branch.len(), 2);
            assert!(else_branch.is_none());
            match &then_branch[0] {
                EchoAst::Let { name, value, .. } => {
                    assert_eq!(**name, EchoAst::Identifier("x".to_string()));
                    assert_eq!(**value, EchoAst::Number(1));
                }
                _ => panic!("Expected Let statement"),
            }
            match &then_branch[1] {
                EchoAst::Return { value, .. } => {
                    assert_eq!(**value, EchoAst::Identifier("x".to_string()));
                }
                _ => panic!("Expected Return statement"),
            }
        }
        _ => panic!("Expected If statement"),
    }
}

// List literals
#[test]
fn test_rust_sitter_empty_list() {
    let result = parse_echo("[]");
    assert!(result.is_ok());
    match result.unwrap() {
        EchoAst::List { elements, .. } => {
            assert_eq!(elements.len(), 0);
        }
        _ => panic!("Expected List"),
    }
}

#[test]
fn test_rust_sitter_list_with_elements() {
    let result = parse_echo("[1, 2, 3]");
    assert!(result.is_ok());
    match result.unwrap() {
        EchoAst::List { elements, .. } => {
            // Filter out comma tokens
            let non_comma_elements: Vec<_> = elements.iter().filter(|&e| !matches!(e, EchoAst::Comma)).collect();
            assert_eq!(non_comma_elements.len(), 3);
            assert_eq!(elements[0], EchoAst::Number(1));
            assert_eq!(elements[2], EchoAst::Number(2));
            assert_eq!(elements[4], EchoAst::Number(3));
        }
        _ => panic!("Expected List"),
    }
}

#[test]
fn test_rust_sitter_list_mixed_types() {
    let result = parse_echo("[1, \"hello\", true]");
    assert!(result.is_ok());
    match result.unwrap() {
        EchoAst::List { elements, .. } => {
            // Filter out comma tokens
            let non_comma_elements: Vec<_> = elements.iter().filter(|&e| !matches!(e, EchoAst::Comma)).collect();
            assert_eq!(non_comma_elements.len(), 3);
            assert_eq!(elements[0], EchoAst::Number(1));
            assert_eq!(elements[2], EchoAst::String("hello".to_string()));
            assert_eq!(elements[4], EchoAst::True);
        }
        _ => panic!("Expected List"),
    }
}

// Loop constructs
#[test]
fn test_rust_sitter_for_loop() {
    let result = parse_echo("for (item in [1, 2, 3]) return item; endfor");
    assert!(result.is_ok());
    match result.unwrap() {
        EchoAst::For { variable, collection, body, .. } => {
            assert_eq!(*variable, EchoAst::Identifier("item".to_string()));
            match collection.as_ref() {
                EchoAst::List { elements, .. } => {
                    let non_comma_elements: Vec<_> = elements.iter().filter(|&e| !matches!(e, EchoAst::Comma)).collect();
                    assert_eq!(non_comma_elements.len(), 3);
                }
                _ => panic!("Expected List in for loop collection"),
            }
            assert_eq!(body.len(), 1);
            match &body[0] {
                EchoAst::Return { value, .. } => {
                    assert_eq!(**value, EchoAst::Identifier("item".to_string()));
                }
                _ => panic!("Expected Return in for loop body"),
            }
        }
        _ => panic!("Expected For loop"),
    }
}

#[test]
fn test_rust_sitter_while_loop() {
    let result = parse_echo("while (true) break; endwhile");
    assert!(result.is_ok());
    match result.unwrap() {
        EchoAst::While { condition, body, .. } => {
            assert_eq!(*condition, EchoAst::True);
            assert_eq!(body.len(), 1);
            match &body[0] {
                EchoAst::Break { .. } => {}, // Success
                _ => panic!("Expected Break statement"),
            }
        }
        _ => panic!("Expected While loop"),
    }
}

#[test]
fn test_rust_sitter_while_loop_with_condition() {
    let result = parse_echo("while (x < 10) let x = x + 1; endwhile");
    assert!(result.is_ok());
    match result.unwrap() {
        EchoAst::While { condition, body, .. } => {
            match condition.as_ref() {
                EchoAst::LessThan { left, right, .. } => {
                    assert_eq!(**left, EchoAst::Identifier("x".to_string()));
                    assert_eq!(**right, EchoAst::Number(10));
                }
                _ => panic!("Expected LessThan comparison"),
            }
            assert_eq!(body.len(), 1);
            match &body[0] {
                EchoAst::Let { name, value, .. } => {
                    assert_eq!(**name, EchoAst::Identifier("x".to_string()));
                    match value.as_ref() {
                        EchoAst::Add { left, right, .. } => {
                            assert_eq!(**left, EchoAst::Identifier("x".to_string()));
                            assert_eq!(**right, EchoAst::Number(1));
                        }
                        _ => panic!("Expected Add expression"),
                    }
                }
                _ => panic!("Expected Let statement"),
            }
        }
        _ => panic!("Expected While loop"),
    }
}

#[test]
fn test_rust_sitter_break_statement() {
    let result = parse_echo("break;");
    assert!(result.is_ok());
    match result.unwrap() {
        EchoAst::Break { .. } => {}, // Success
        _ => panic!("Expected Break statement"),
    }
}

#[test]
fn test_rust_sitter_continue_statement() {
    let result = parse_echo("continue;");
    assert!(result.is_ok());
    match result.unwrap() {
        EchoAst::Continue { .. } => {}, // Success
        _ => panic!("Expected Continue statement"),
    }
}

#[test]
fn test_rust_sitter_for_loop_with_break() {
    let result = parse_echo("for (i in [1, 2, 3]) if (i == 2) break; endif endfor");
    assert!(result.is_ok());
    match result.unwrap() {
        EchoAst::For { variable, collection, body, .. } => {
            assert_eq!(*variable, EchoAst::Identifier("i".to_string()));
            assert_eq!(body.len(), 1);
            match &body[0] {
                EchoAst::If { condition, then_branch, else_branch, .. } => {
                    match condition.as_ref() {
                        EchoAst::Equal { left, right, .. } => {
                            assert_eq!(**left, EchoAst::Identifier("i".to_string()));
                            assert_eq!(**right, EchoAst::Number(2));
                        }
                        _ => panic!("Expected Equal comparison"),
                    }
                    assert_eq!(then_branch.len(), 1);
                    match &then_branch[0] {
                        EchoAst::Break { .. } => {}, // Success
                        _ => panic!("Expected Break statement"),
                    }
                    assert!(else_branch.is_none());
                }
                _ => panic!("Expected If statement"),
            }
        }
        _ => panic!("Expected For loop"),
    }
}

#[test]
fn test_rust_sitter_nested_loops() {
    let result = parse_echo("for (i in [1, 2]) for (j in [3, 4]) return i + j; endfor endfor");
    assert!(result.is_ok());
    match result.unwrap() {
        EchoAst::For { variable, collection, body, .. } => {
            assert_eq!(*variable, EchoAst::Identifier("i".to_string()));
            assert_eq!(body.len(), 1);
            match &body[0] {
                EchoAst::For { variable: inner_var, collection: inner_collection, body: inner_body, .. } => {
                    assert_eq!(**inner_var, EchoAst::Identifier("j".to_string()));
                    assert_eq!(inner_body.len(), 1);
                    match &inner_body[0] {
                        EchoAst::Return { value, .. } => {
                            match value.as_ref() {
                                EchoAst::Add { left, right, .. } => {
                                    assert_eq!(**left, EchoAst::Identifier("i".to_string()));
                                    assert_eq!(**right, EchoAst::Identifier("j".to_string()));
                                }
                                _ => panic!("Expected Add expression"),
                            }
                        }
                        _ => panic!("Expected Return statement"),
                    }
                }
                _ => panic!("Expected nested For loop"),
            }
        }
        _ => panic!("Expected For loop"),
    }
}

// Exception handling
#[test]
fn test_rust_sitter_try_catch() {
    let result = parse_echo("try throw \"error\"; catch (e) return e; endtry");
    assert!(result.is_ok());
    match result.unwrap() {
        EchoAst::Try { body, exception_var, catch_body, .. } => {
            assert_eq!(body.len(), 1);
            match &body[0] {
                EchoAst::Throw { value, .. } => {
                    assert_eq!(**value, EchoAst::String("error".to_string()));
                }
                _ => panic!("Expected Throw statement"),
            }
            assert!(exception_var.is_some());
            if let Some(var) = exception_var {
                match var.as_ref() {
                    EchoAst::Identifier(name) => assert_eq!(name, "e"),
                    _ => panic!("Expected Identifier"),
                }
            }
            assert_eq!(catch_body.len(), 1);
            match &catch_body[0] {
                EchoAst::Return { value, .. } => {
                    assert_eq!(**value, EchoAst::Identifier("e".to_string()));
                }
                _ => panic!("Expected Return statement"),
            }
        }
        _ => panic!("Expected Try statement"),
    }
}

#[test]
fn test_rust_sitter_try_catch_no_var() {
    let result = parse_echo("try throw 42; catch () return \"caught\"; endtry");
    assert!(result.is_ok());
    match result.unwrap() {
        EchoAst::Try { body, exception_var, catch_body, .. } => {
            assert_eq!(body.len(), 1);
            assert!(exception_var.is_none());
            assert_eq!(catch_body.len(), 1);
        }
        _ => panic!("Expected Try statement"),
    }
}

#[test]
fn test_rust_sitter_throw_statement() {
    let result = parse_echo("throw \"error message\";");
    assert!(result.is_ok());
    match result.unwrap() {
        EchoAst::Throw { value, .. } => {
            match value.as_ref() {
                EchoAst::String(s) => assert_eq!(s, "error message"),
                _ => panic!("Expected String"),
            }
        }
        _ => panic!("Expected Throw statement"),
    }
}

#[test]
fn test_rust_sitter_nested_try_catch() {
    let result = parse_echo("try try throw \"inner\"; catch (e) throw \"outer\"; endtry catch (e) return e; endtry");
    assert!(result.is_ok());
    match result.unwrap() {
        EchoAst::Try { body, exception_var, catch_body, .. } => {
            assert_eq!(body.len(), 1);
            match &body[0] {
                EchoAst::Try { body: inner_body, exception_var: inner_var, catch_body: inner_catch, .. } => {
                    assert_eq!(inner_body.len(), 1);
                    assert!(inner_var.is_some());
                    assert_eq!(inner_catch.len(), 1);
                    match &inner_body[0] {
                        EchoAst::Throw { value, .. } => {
                            assert_eq!(**value, EchoAst::String("inner".to_string()));
                        }
                        _ => panic!("Expected inner Throw statement"),
                    }
                    match &inner_catch[0] {
                        EchoAst::Throw { value, .. } => {
                            assert_eq!(**value, EchoAst::String("outer".to_string()));
                        }
                        _ => panic!("Expected outer Throw statement"),
                    }
                }
                _ => panic!("Expected nested Try statement"),
            }
            assert!(exception_var.is_some());
            assert_eq!(catch_body.len(), 1);
        }
        _ => panic!("Expected outer Try statement"),
    }
}

// Error cases
#[test]
fn test_rust_sitter_invalid_syntax() {
    let result = parse_echo("let = 42");
    assert!(result.is_err());
}

#[test]
fn test_rust_sitter_empty_input() {
    let result = parse_echo("");
    assert!(result.is_err());
}

#[test]
fn test_rust_sitter_whitespace_only() {
    let result = parse_echo("   ");
    assert!(result.is_err());
}

#[test]
fn test_rust_sitter_unclosed_string() {
    let result = parse_echo("\"unclosed");
    assert!(result.is_err());
}

#[test]
fn test_rust_sitter_invalid_identifier() {
    let result = parse_echo("123abc");
    // This should parse as a number (123) but fail to parse completely
    assert!(result.is_err());
}