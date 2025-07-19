use echo_repl::parser::EchoParser;
use echo_repl::evaluator::{create_evaluator, create_evaluator_of_type, EvaluatorType, EvaluatorTrait, Value};
use echo_repl::storage::Storage;
use std::sync::Arc;

#[cfg(feature = "wasm-jit")]
#[test]
fn test_wasm_jit_evaluator_creation() {
    let storage = Arc::new(Storage::new("./test-wasm-jit-creation-db").unwrap());
    let evaluator = create_evaluator_of_type(storage, EvaluatorType::WasmJit);
    assert!(evaluator.is_ok());
}

#[cfg(feature = "wasm-jit")]
#[test]
fn test_wasm_jit_basic_number() {
    let storage = Arc::new(Storage::new("./test-wasm-jit-number-db").unwrap());
    let mut evaluator = create_evaluator_of_type(storage, EvaluatorType::WasmJit).unwrap();
    
    let mut parser = EchoParser::new().unwrap();
    let ast = parser.parse("42").unwrap();
    
    let player_id = evaluator.create_player("test").unwrap();
    let result = evaluator.eval_with_player(&ast, player_id).unwrap();
    
    assert_eq!(result, Value::Integer(42));
}

#[cfg(feature = "wasm-jit")]
#[test]
fn test_wasm_jit_basic_arithmetic() {
    let storage = Arc::new(Storage::new("./test-wasm-jit-arithmetic-db").unwrap());
    let mut evaluator = create_evaluator_of_type(storage, EvaluatorType::WasmJit).unwrap();
    
    let mut parser = EchoParser::new().unwrap();
    let ast = parser.parse("2 + 3").unwrap();
    
    let player_id = evaluator.create_player("test").unwrap();
    let result = evaluator.eval_with_player(&ast, player_id).unwrap();
    
    assert_eq!(result, Value::Integer(5));
}

#[cfg(feature = "wasm-jit")]
#[test]
fn test_wasm_jit_complex_arithmetic() {
    let storage = Arc::new(Storage::new("./test-wasm-jit-complex-db").unwrap());
    let mut evaluator = create_evaluator_of_type(storage, EvaluatorType::WasmJit).unwrap();
    
    let mut parser = EchoParser::new().unwrap();
    let ast = parser.parse("10 + 20 + 30").unwrap();
    
    let player_id = evaluator.create_player("test").unwrap();
    let result = evaluator.eval_with_player(&ast, player_id).unwrap();
    
    assert_eq!(result, Value::Integer(60));
}

#[cfg(feature = "wasm-jit")]
#[test]
fn test_wasm_jit_string_literal() {
    let storage = Arc::new(Storage::new("./test-wasm-jit-string-db").unwrap());
    let mut evaluator = create_evaluator_of_type(storage, EvaluatorType::WasmJit).unwrap();
    
    let mut parser = EchoParser::new().unwrap();
    let ast = parser.parse(r#""hello world""#).unwrap();
    
    let player_id = evaluator.create_player("test").unwrap();
    let result = evaluator.eval_with_player(&ast, player_id).unwrap();
    
    assert_eq!(result, Value::String("hello world".to_string()));
}

#[cfg(feature = "wasm-jit")]
#[test]
fn test_wasm_jit_string_concatenation() {
    let storage = Arc::new(Storage::new("./test-wasm-jit-concat-db").unwrap());
    let mut evaluator = create_evaluator_of_type(storage, EvaluatorType::WasmJit).unwrap();
    
    let mut parser = EchoParser::new().unwrap();
    let ast = parser.parse(r#""hello" + " world""#).unwrap();
    
    let player_id = evaluator.create_player("test").unwrap();
    let result = evaluator.eval_with_player(&ast, player_id).unwrap();
    
    assert_eq!(result, Value::String("hello world".to_string()));
}

#[cfg(feature = "wasm-jit")]
#[test]
fn test_wasm_jit_vs_interpreter_compatibility() {
    let storage = Arc::new(Storage::new("./test-wasm-jit-compatibility-db").unwrap());
    
    // Test the same expression with both evaluators
    let test_cases = vec![
        "42",
        "2 + 3",
        "10 + 20 + 30",
        r#""hello""#,
        r#""hello" + " world""#,
    ];
    
    for expr in test_cases {
        let mut parser = EchoParser::new().unwrap();
        let ast = parser.parse(expr).unwrap();
        
        // Test with interpreter
        let mut interpreter = create_evaluator_of_type(storage.clone(), EvaluatorType::Interpreter).unwrap();
        let player_id1 = interpreter.create_player("test").unwrap();
        let result1 = interpreter.eval_with_player(&ast, player_id1).unwrap();
        
        // Test with WebAssembly JIT
        let mut wasm_jit = create_evaluator_of_type(storage.clone(), EvaluatorType::WasmJit).unwrap();
        let player_id2 = wasm_jit.create_player("test").unwrap();
        let result2 = wasm_jit.eval_with_player(&ast, player_id2).unwrap();
        
        assert_eq!(result1, result2, "Results differ for expression: {}", expr);
    }
}

#[cfg(feature = "wasm-jit")]
#[test]
fn test_wasm_jit_performance_behavior() {
    let storage = Arc::new(Storage::new("./test-wasm-jit-perf-db").unwrap());
    let mut evaluator = create_evaluator_of_type(storage, EvaluatorType::WasmJit).unwrap();
    
    // Test that the evaluator can handle repeated evaluations
    let mut parser = EchoParser::new().unwrap();
    let ast = parser.parse("2 + 3").unwrap();
    
    let player_id = evaluator.create_player("test").unwrap();
    
    // Run the same expression multiple times
    for _i in 0..10 {
        let result = evaluator.eval_with_player(&ast, player_id).unwrap();
        assert_eq!(result, Value::Integer(5));
    }
}

#[test]
fn test_wasm_jit_feature_fallback() {
    let storage = Arc::new(Storage::new("./test-wasm-jit-fallback-db").unwrap());
    
    // This should work regardless of feature flags
    let evaluator = create_evaluator(storage);
    assert!(evaluator.is_ok());
}

#[test]
fn test_evaluator_trait_polymorphism_with_wasm_jit() {
    fn test_evaluator(mut evaluator: Box<dyn EvaluatorTrait>) {
        let mut parser = EchoParser::new().unwrap();
        let ast = parser.parse("42").unwrap();
        
        let player_id = evaluator.create_player("test").unwrap();
        let result = evaluator.eval_with_player(&ast, player_id).unwrap();
        
        assert_eq!(result, Value::Integer(42));
    }
    
    let storage = Arc::new(Storage::new("./test-wasm-jit-poly-db").unwrap());
    
    // Test interpreter
    let interpreter = create_evaluator_of_type(storage.clone(), EvaluatorType::Interpreter).unwrap();
    test_evaluator(interpreter);
    
    // Test Cranelift JIT if available
    #[cfg(feature = "jit")]
    {
        let jit = create_evaluator_of_type(storage.clone(), EvaluatorType::Jit).unwrap();
        test_evaluator(jit);
    }
    
    // Test WebAssembly JIT if available
    #[cfg(feature = "wasm-jit")]
    {
        let wasm_jit = create_evaluator_of_type(storage.clone(), EvaluatorType::WasmJit).unwrap();
        test_evaluator(wasm_jit);
    }
}