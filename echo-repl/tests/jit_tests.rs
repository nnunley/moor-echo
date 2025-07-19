use echo_repl::parser::EchoParser;
use echo_repl::evaluator::{create_evaluator, create_evaluator_of_type, EvaluatorType, EvaluatorTrait, Value};
use echo_repl::storage::Storage;
use std::sync::Arc;

#[test]
fn test_factory_evaluator() {
    let storage = Arc::new(Storage::new("./test-factory-db").unwrap());
    let mut evaluator = create_evaluator(storage).unwrap();
    
    // Test basic functionality
    let mut parser = EchoParser::new().unwrap();
    let ast = parser.parse("42").unwrap();
    
    let player_id = evaluator.create_player("test").unwrap();
    let result = evaluator.eval_with_player(&ast, player_id).unwrap();
    
    assert_eq!(result, Value::Integer(42));
}

#[test]
fn test_interpreter_evaluator() {
    let storage = Arc::new(Storage::new("./test-interpreter-db").unwrap());
    let mut evaluator = create_evaluator_of_type(storage, EvaluatorType::Interpreter).unwrap();
    
    // Test string literal
    let mut parser = EchoParser::new().unwrap();
    let ast = parser.parse(r#""hello world""#).unwrap();
    
    let player_id = evaluator.create_player("test").unwrap();
    let result = evaluator.eval_with_player(&ast, player_id).unwrap();
    
    assert_eq!(result, Value::String("hello world".to_string()));
}

#[cfg(feature = "jit")]
#[test]
fn test_jit_evaluator() {
    let storage = Arc::new(Storage::new("./test-jit-db").unwrap());
    let mut evaluator = create_evaluator_of_type(storage, EvaluatorType::Jit).unwrap();
    
    // Test basic arithmetic
    let mut parser = EchoParser::new().unwrap();
    let ast = parser.parse("2 + 3").unwrap();
    
    let player_id = evaluator.create_player("test").unwrap();
    let result = evaluator.eval_with_player(&ast, player_id).unwrap();
    
    assert_eq!(result, Value::Integer(5));
}

#[cfg(feature = "jit")]
#[test]
fn test_jit_string_concatenation() {
    let storage = Arc::new(Storage::new("./test-jit-concat-db").unwrap());
    let mut evaluator = create_evaluator_of_type(storage, EvaluatorType::Jit).unwrap();
    
    // Test string concatenation
    let mut parser = EchoParser::new().unwrap();
    let ast = parser.parse(r#""hello" + " world""#).unwrap();
    
    let player_id = evaluator.create_player("test").unwrap();
    let result = evaluator.eval_with_player(&ast, player_id).unwrap();
    
    assert_eq!(result, Value::String("hello world".to_string()));
}

#[cfg(feature = "wasm-jit")]
#[test]
fn test_wasm_jit_evaluator_basic() {
    let storage = Arc::new(Storage::new("./test-wasm-jit-basic-db").unwrap());
    let mut evaluator = create_evaluator_of_type(storage, EvaluatorType::WasmJit).unwrap();
    
    // Test basic arithmetic
    let mut parser = EchoParser::new().unwrap();
    let ast = parser.parse("2 + 3").unwrap();
    
    let player_id = evaluator.create_player("test").unwrap();
    let result = evaluator.eval_with_player(&ast, player_id).unwrap();
    
    assert_eq!(result, Value::Integer(5));
}

#[cfg(feature = "wasm-jit")]
#[test]
fn test_wasm_jit_string_concatenation() {
    let storage = Arc::new(Storage::new("./test-wasm-jit-concat-db").unwrap());
    let mut evaluator = create_evaluator_of_type(storage, EvaluatorType::WasmJit).unwrap();
    
    // Test string concatenation
    let mut parser = EchoParser::new().unwrap();
    let ast = parser.parse(r#""hello" + " world""#).unwrap();
    
    let player_id = evaluator.create_player("test").unwrap();
    let result = evaluator.eval_with_player(&ast, player_id).unwrap();
    
    assert_eq!(result, Value::String("hello world".to_string()));
}

#[test]
fn test_evaluator_trait_polymorphism() {
    fn test_evaluator(mut evaluator: Box<dyn EvaluatorTrait>) {
        let mut parser = EchoParser::new().unwrap();
        let ast = parser.parse("42").unwrap();
        
        let player_id = evaluator.create_player("test").unwrap();
        let result = evaluator.eval_with_player(&ast, player_id).unwrap();
        
        assert_eq!(result, Value::Integer(42));
    }
    
    let storage = Arc::new(Storage::new("./test-poly-db").unwrap());
    let interpreter = create_evaluator_of_type(storage.clone(), EvaluatorType::Interpreter).unwrap();
    test_evaluator(interpreter);
    
    #[cfg(feature = "jit")]
    {
        let jit = create_evaluator_of_type(storage.clone(), EvaluatorType::Jit).unwrap();
        test_evaluator(jit);
    }
    
    #[cfg(feature = "wasm-jit")]
    {
        let wasm_jit = create_evaluator_of_type(storage, EvaluatorType::WasmJit).unwrap();
        test_evaluator(wasm_jit);
    }
}