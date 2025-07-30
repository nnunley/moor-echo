use echo_core::{
    parser::create_parser,
    evaluator::{Evaluator, JitEvaluator},
    storage::Storage,
    Value,
};
use std::sync::Arc;
use tempfile::TempDir;

fn test_code(name: &str, code: &str) {
    println!("\n=== Testing: {} ===", name);
    println!("Code: {}", code);
    
    let temp_dir = TempDir::new().unwrap();
    let storage = Arc::new(Storage::new(temp_dir.path()).unwrap());
    
    // Parse the code
    let mut parser = create_parser("echo").unwrap();
    let ast = if code.contains('\n') {
        parser.parse_program(code).unwrap()
    } else {
        parser.parse(code).unwrap()
    };
    
    // Test with interpreter
    let mut interpreter = Evaluator::new(storage.clone());
    let player_id = interpreter.create_player("test").unwrap();
    let interp_result = interpreter.eval_with_player(&ast, player_id).unwrap();
    println!("Interpreter result: {:?}", interp_result);
    
    // Test with JIT
    let mut jit = JitEvaluator::new_with_fallback(storage);
    let player_id = jit.create_player("test").unwrap();
    let jit_result = jit.eval_with_player(&ast, player_id).unwrap();
    println!("JIT result: {:?}", jit_result);
    
    // Compare results
    match (interp_result, jit_result) {
        (Value::Integer(n1), Value::Integer(n2)) if n1 == n2 => {
            println!("✅ Results match!");
        }
        (Value::Boolean(b1), Value::Boolean(b2)) if b1 == b2 => {
            println!("✅ Results match!");
        }
        (v1, v2) if v1 == v2 => {
            println!("✅ Results match!");
        }
        (v1, v2) => {
            println!("❌ Results differ! Interpreter: {:?}, JIT: {:?}", v1, v2);
        }
    }
}

fn main() {
    println!("Testing JIT enhancements...\n");
    
    // Test boolean literals
    test_code("Boolean true", "true");
    test_code("Boolean false", "false");
    
    // Test variable access
    test_code("Simple variable", r#"
let x = 5
x
    "#);
    
    test_code("Variable arithmetic", r#"
let x = 10
let y = 20
x + y
    "#);
    
    // Test if statements
    test_code("Simple if true", r#"
if (true)
    42
else
    0
endif
    "#);
    
    test_code("Simple if false", r#"
if (false)
    42
else
    99
endif
    "#);
    
    test_code("If with comparison", r#"
if (5 < 10)
    100
else
    200
endif
    "#);
    
    test_code("If with variable condition", r#"
let x = 15
if (x > 10)
    x * 2
else
    x / 2
endif
    "#);
    
    // Test logical operations
    test_code("Logical AND true", "true && true");
    test_code("Logical AND false", "true && false");
    test_code("Logical OR true", "true || false");
    test_code("Logical OR false", "false || false");
    
    // Test short-circuit evaluation
    test_code("AND short-circuit", r#"
let x = 5
false && (x > 10)
    "#);
    
    test_code("OR short-circuit", r#"
let x = 5
true || (x > 10)
    "#);
    
    // Test complex conditions
    test_code("Complex condition", r#"
let x = 5
let y = 10
(x < y) && (y > 0)
    "#);
    
    // Test nested if statements
    test_code("Nested if", r#"
let x = 15
if (x > 10)
    if (x < 20)
        x * 2
    else
        x * 3
    endif
else
    x / 2
endif
    "#);
    
    // Test multiple statements
    test_code("Multiple statements", r#"
let x = 5
let y = 10
let z = x + y
z * 2
    "#);
}