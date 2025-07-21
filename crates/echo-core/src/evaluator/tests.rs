use super::*;
use crate::parser::create_parser;
use tempfile::TempDir;

fn create_test_evaluator() -> (Evaluator, ObjectId, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let storage = Arc::new(Storage::new(temp_dir.path()).unwrap());
    let mut evaluator = Evaluator::new(storage);
    let player_id = evaluator.create_player("test").unwrap();
    (evaluator, player_id, temp_dir)
}

#[test]
fn test_break_in_while_loop() {
    let (mut evaluator, player_id, _temp_dir) = create_test_evaluator();
    let mut parser = create_parser("echo").unwrap();
    
    // Test simple break
    let code = r#"
let count = 0
while true
  count = count + 1
  if count == 3
    break
  endif
endwhile
count
"#;
    
    let ast = parser.parse_program(code).unwrap();
    let result = evaluator.eval_with_player(&ast, player_id).unwrap();
    assert_eq!(result, Value::Integer(3));
}

#[test] 
fn test_continue_in_while_loop() {
    let (mut evaluator, player_id, _temp_dir) = create_test_evaluator();
    let mut parser = create_parser("echo").unwrap();
    
    // Test continue - skip when i == 3
    let code = r#"
let sum = 0
let i = 0
while i < 5
  i = i + 1
  if i == 3
    continue
  endif
  sum = sum + i
endwhile
sum
"#;
    
    let ast = parser.parse_program(code).unwrap();
    let result = evaluator.eval_with_player(&ast, player_id).unwrap();
    // Should be 1 + 2 + 4 + 5 = 12 (skipping 3)
    assert_eq!(result, Value::Integer(12));
}

#[test]
fn test_break_in_for_loop() {
    let (mut evaluator, player_id, _temp_dir) = create_test_evaluator();
    let mut parser = create_parser("echo").unwrap();
    
    // Test break in for loop
    let code = r#"
let result = 0
for x in [1, 2, 3, 4, 5]
  if x == 3
    break
  endif
  result = result + x
endfor
result
"#;
    
    let ast = parser.parse_program(code).unwrap();
    let result = evaluator.eval_with_player(&ast, player_id).unwrap();
    // Should be 1 + 2 = 3 (stops at 3)
    assert_eq!(result, Value::Integer(3));
}

#[test]
fn test_continue_in_for_loop() {
    let (mut evaluator, player_id, _temp_dir) = create_test_evaluator();
    let mut parser = create_parser("echo").unwrap();
    
    // Test continue in for loop  
    let code = r#"
let sum = 0
for y in [1, 2, 3, 4, 5]
  if y == 3
    continue
  endif
  sum = sum + y
endfor
sum
"#;
    
    let ast = parser.parse_program(code).unwrap();
    let result = evaluator.eval_with_player(&ast, player_id).unwrap();
    // Should be 1 + 2 + 4 + 5 = 12 (skipping 3)
    assert_eq!(result, Value::Integer(12));
}

#[test]
fn test_nested_loops_with_break() {
    let (mut evaluator, player_id, _temp_dir) = create_test_evaluator();
    let mut parser = create_parser("echo").unwrap();
    
    // Test nested loops
    let code = r#"
let outer = 0
let inner_total = 0
while outer < 3
  outer = outer + 1
  let inner = 0
  while inner < 5
    inner = inner + 1
    if inner == 3
      break
    endif
    inner_total = inner_total + 1
  endwhile
endwhile
inner_total
"#;
    
    let ast = parser.parse_program(code).unwrap();
    let result = evaluator.eval_with_player(&ast, player_id).unwrap();
    // Each outer loop iteration contributes 2 to inner_total (breaks at inner == 3)
    // So 3 * 2 = 6
    assert_eq!(result, Value::Integer(6));
}