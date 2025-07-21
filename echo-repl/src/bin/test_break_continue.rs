use echo_repl::parser::create_parser;
use echo_repl::evaluator::Evaluator;
use echo_repl::storage::Storage;
use std::sync::Arc;

fn main() -> anyhow::Result<()> {
    let storage = Arc::new(Storage::new("./echo-test-db")?);
    let mut evaluator = Evaluator::new(storage);
    let player_id = evaluator.create_player("test")?;
    let mut parser = create_parser("echo")?;
    
    println!("Testing break/continue functionality\n");
    
    // Test 1: Simple break
    println!("Test 1: Simple break in while loop");
    let code1 = r#"
let count = 0
count
"#;
    match parser.parse_program(code1) {
        Ok(ast) => {
            let result = evaluator.eval_with_player(&ast, player_id)?;
            println!("Initial count: {:?}", result);
        }
        Err(e) => println!("Parse error: {:?}", e),
    }
    
    // Test while loop separately
    let code2 = r#"
while count < 10
  count = count + 1
  if count == 3
    break
  endif
endwhile
"#;
    match parser.parse_program(code2) {
        Ok(ast) => {
            let result = evaluator.eval_with_player(&ast, player_id)?;
            println!("After while loop: {:?}", result);
        }
        Err(e) => println!("Parse error in while loop: {:?}", e),
    }
    
    // Check final count
    let code3 = "count";
    match parser.parse(code3) {
        Ok(ast) => {
            let result = evaluator.eval_with_player(&ast, player_id)?;
            println!("Final count: {:?}", result);
        }
        Err(e) => println!("Parse error: {:?}", e),
    }
    
    // Test 2: Continue in while loop
    println!("\nTest 2: Continue in while loop");
    let setup = r#"
let sum = 0
let i = 0
"#;
    parser.parse_program(setup).map(|ast| evaluator.eval_with_player(&ast, player_id)).ok();
    
    let loop_code = r#"
while i < 5
  i = i + 1
  if i == 3
    continue
  endif
  sum = sum + i
endwhile
"#;
    match parser.parse_program(loop_code) {
        Ok(ast) => {
            evaluator.eval_with_player(&ast, player_id)?;
            // Check sum
            let sum_ast = parser.parse("sum")?;
            let sum_result = evaluator.eval_with_player(&sum_ast, player_id)?;
            println!("Sum (should be 12): {:?}", sum_result);
        }
        Err(e) => println!("Parse error in continue test: {:?}", e),
    }
    
    Ok(())
}