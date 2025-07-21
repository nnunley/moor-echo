use echo_repl::parser::create_parser;
use echo_repl::evaluator::Evaluator;
use echo_repl::storage::Storage;
use std::sync::Arc;

fn main() -> anyhow::Result<()> {
    let storage = Arc::new(Storage::new("./echo-test-db")?);
    let mut evaluator = Evaluator::new(storage);
    let player_id = evaluator.create_player("test")?;
    let mut parser = create_parser("echo")?;
    
    println!("Testing lambda parameters functionality\n");
    
    // Test 1: Simple parameters (should work)
    println!("Test 1: Simple parameters");
    let code1 = "let f = fn {x, y} x + y endfn";
    match parser.parse(code1) {
        Ok(ast) => {
            println!("Parsed successfully");
            evaluator.eval_with_player(&ast, player_id)?;
            
            // Call it
            let call = parser.parse("f(5, 3)")?;
            let result = evaluator.eval_with_player(&call, player_id)?;
            println!("f(5, 3) = {:?}\n", result);
        }
        Err(e) => println!("Parse error: {:?}\n", e),
    }
    
    // Test 2: Optional parameters
    println!("Test 2: Optional parameters");
    let code2 = "fn {x, ?y=10} x + y endfn";
    match parser.parse(code2) {
        Ok(ast) => {
            println!("Parsed successfully");
            let result = evaluator.eval_with_player(&ast, player_id)?;
            println!("Lambda created: {:?}\n", result);
        }
        Err(e) => println!("Parse error: {:?}\n", e),
    }
    
    // Test 3: Rest parameters
    println!("Test 3: Rest parameters");
    let code3 = "fn {x, @rest} rest endfn";
    match parser.parse(code3) {
        Ok(ast) => {
            println!("Parsed successfully");
            let result = evaluator.eval_with_player(&ast, player_id)?;
            println!("Lambda created: {:?}\n", result);
        }
        Err(e) => println!("Parse error: {:?}\n", e),
    }
    
    // Test 4: Test optional parameter evaluation
    println!("Test 4: Testing optional parameter evaluation");
    let setup = parser.parse("let opt_func = fn {x, ?y=5} x + y endfn")?;
    evaluator.eval_with_player(&setup, player_id)?;
    
    // Call with default
    let call1 = parser.parse("opt_func(10)")?;
    let result1 = evaluator.eval_with_player(&call1, player_id)?;
    println!("opt_func(10) = {:?} (should be 15)", result1);
    
    // Call with both args
    let call2 = parser.parse("opt_func(10, 20)")?;
    let result2 = evaluator.eval_with_player(&call2, player_id)?;
    println!("opt_func(10, 20) = {:?} (should be 30)\n", result2);
    
    // Test 5: Test rest parameters
    println!("Test 5: Testing rest parameter evaluation");
    let rest_setup = parser.parse("let rest_func = fn {x, @rest} rest endfn")?;
    evaluator.eval_with_player(&rest_setup, player_id)?;
    
    let rest_call = parser.parse("rest_func(1, 2, 3, 4, 5)")?;
    let rest_result = evaluator.eval_with_player(&rest_call, player_id)?;
    println!("rest_func(1, 2, 3, 4, 5) = {:?} (should be [2, 3, 4, 5])", rest_result);
    
    // Test 6: Mixed parameters
    println!("\nTest 6: Testing mixed parameters");
    let mixed_setup = parser.parse("let mixed = fn {x, ?y=10, @rest} [x, y, rest] endfn")?;
    evaluator.eval_with_player(&mixed_setup, player_id)?;
    
    let mixed1 = parser.parse("mixed(1)")?;
    let r1 = evaluator.eval_with_player(&mixed1, player_id)?;
    println!("mixed(1) = {:?} (should be [1, 10, []])", r1);
    
    let mixed2 = parser.parse("mixed(1, 2)")?;
    let r2 = evaluator.eval_with_player(&mixed2, player_id)?;
    println!("mixed(1, 2) = {:?} (should be [1, 2, []])", r2);
    
    let mixed3 = parser.parse("mixed(1, 2, 3, 4, 5)")?;
    let r3 = evaluator.eval_with_player(&mixed3, player_id)?;
    println!("mixed(1, 2, 3, 4, 5) = {:?} (should be [1, 2, [3, 4, 5]])", r3);
    
    Ok(())
}