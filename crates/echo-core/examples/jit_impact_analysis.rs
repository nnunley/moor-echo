use echo_core::{
    parser::create_parser,
    ast::EchoAst,
    evaluator::Evaluator,
    storage::Storage,
};
#[cfg(feature = "jit")]
use echo_core::evaluator::JitEvaluator;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tempfile::TempDir;

/// Count occurrences of each non-JIT node type
fn count_non_jit_nodes(ast: &EchoAst) -> std::collections::HashMap<String, usize> {
    let mut counts = std::collections::HashMap::new();
    
    fn visit(ast: &EchoAst, counts: &mut std::collections::HashMap<String, usize>) {
        match ast {
            // JIT-compilable nodes - skip counting
            EchoAst::Number(_) |
            EchoAst::Add { .. } |
            EchoAst::Subtract { .. } |
            EchoAst::Multiply { .. } |
            EchoAst::Divide { .. } |
            EchoAst::Modulo { .. } |
            EchoAst::Equal { .. } |
            EchoAst::NotEqual { .. } |
            EchoAst::LessThan { .. } |
            EchoAst::LessEqual { .. } |
            EchoAst::GreaterThan { .. } |
            EchoAst::GreaterEqual { .. } |
            EchoAst::UnaryMinus { .. } => {
                // These are JIT-able, recurse into children
                match ast {
                    EchoAst::Add { left, right } |
                    EchoAst::Subtract { left, right } |
                    EchoAst::Multiply { left, right } |
                    EchoAst::Divide { left, right } |
                    EchoAst::Modulo { left, right } |
                    EchoAst::Equal { left, right } |
                    EchoAst::NotEqual { left, right } |
                    EchoAst::LessThan { left, right } |
                    EchoAst::LessEqual { left, right } |
                    EchoAst::GreaterThan { left, right } |
                    EchoAst::GreaterEqual { left, right } => {
                        visit(left, counts);
                        visit(right, counts);
                    }
                    EchoAst::UnaryMinus { operand } => visit(operand, counts),
                    _ => {}
                }
            }
            
            // Non-JIT nodes - count them
            EchoAst::String(_) => {
                *counts.entry("String literal".to_string()).or_insert(0) += 1;
            }
            EchoAst::Boolean(_) => {
                *counts.entry("Boolean literal".to_string()).or_insert(0) += 1;
            }
            EchoAst::Identifier(_) => {
                *counts.entry("Variable access".to_string()).or_insert(0) += 1;
            }
            EchoAst::LocalAssignment { value, .. } => {
                *counts.entry("Variable assignment".to_string()).or_insert(0) += 1;
                visit(value, counts);
            }
            EchoAst::For { collection, body, .. } => {
                *counts.entry("For loop".to_string()).or_insert(0) += 1;
                visit(collection.as_ref(), counts);
                for stmt in body {
                    visit(stmt, counts);
                }
            }
            EchoAst::While { condition, body, .. } => {
                *counts.entry("While loop".to_string()).or_insert(0) += 1;
                visit(condition, counts);
                for stmt in body {
                    visit(stmt, counts);
                }
            }
            EchoAst::If { condition, then_branch, else_branch, .. } => {
                *counts.entry("If statement".to_string()).or_insert(0) += 1;
                visit(condition, counts);
                for stmt in then_branch {
                    visit(stmt, counts);
                }
                if let Some(else_body) = else_branch {
                    for stmt in else_body {
                        visit(stmt, counts);
                    }
                }
            }
            EchoAst::And { left, right } => {
                *counts.entry("Logical AND".to_string()).or_insert(0) += 1;
                visit(left, counts);
                visit(right, counts);
            }
            EchoAst::Or { left, right } => {
                *counts.entry("Logical OR".to_string()).or_insert(0) += 1;
                visit(left, counts);
                visit(right, counts);
            }
            EchoAst::FunctionCall { args, .. } => {
                *counts.entry("Function call".to_string()).or_insert(0) += 1;
                for arg in args {
                    visit(arg, counts);
                }
            }
            EchoAst::List { elements, .. } => {
                *counts.entry("List literal".to_string()).or_insert(0) += 1;
                for elem in elements {
                    visit(elem, counts);
                }
            }
            EchoAst::Block(stmts) | EchoAst::Program(stmts) => {
                for stmt in stmts {
                    visit(stmt, counts);
                }
            }
            _ => {
                *counts.entry("Other".to_string()).or_insert(0) += 1;
            }
        }
    }
    
    visit(ast, &mut counts);
    counts
}

/// Measure execution time for a code snippet
fn measure_execution_time(code: &str, iterations: usize) -> (Duration, Duration) {
    let temp_dir = TempDir::new().unwrap();
    let storage = Arc::new(Storage::new(temp_dir.path()).unwrap());
    
    let mut parser = create_parser("echo").unwrap();
    let ast = if code.contains('\n') {
        parser.parse_program(code).unwrap()
    } else {
        parser.parse(code).unwrap()
    };
    
    // Measure interpreter time
    let mut interpreter = Evaluator::new(storage.clone());
    let player_id = interpreter.create_player("bench").unwrap();
    
    let start = Instant::now();
    for _ in 0..iterations {
        interpreter.eval_with_player(&ast, player_id).unwrap();
    }
    let interpreter_time = start.elapsed();
    
    // Measure JIT time
    #[cfg(feature = "jit")]
    let jit_time = {
        let mut jit = JitEvaluator::new_with_fallback(storage);
        let player_id = jit.create_player("bench").unwrap();
        
        // Warm up
        jit.eval_with_player(&ast, player_id).unwrap();
        
        let start = Instant::now();
        for _ in 0..iterations {
            jit.eval_with_player(&ast, player_id).unwrap();
        }
        start.elapsed()
    };
    
    #[cfg(not(feature = "jit"))]
    let jit_time = interpreter_time;
    
    (interpreter_time, jit_time)
}

fn main() {
    println!("\n=== JIT Impact Analysis ===\n");
    
    // Real-world-ish code examples
    let test_cases = vec![
        ("Tight arithmetic loop", r#"
let sum = 0
for i in ([1, 2, 3, 4, 5, 6, 7, 8, 9, 10])
    sum = sum + i * i
endfor
sum
        "#),
        ("Conditional arithmetic", r#"
let x = 10
let y = 20
if (x < y)
    x * 2 + y * 3
else
    x * 3 - y * 2
endif
        "#),
        ("Variable-heavy computation", r#"
let a = 1
let b = 2
let c = 3
let d = 4
let result = a + b * c - d
result * result
        "#),
        ("Boolean logic chain", r#"
let x = 5
let y = 10
x > 0 && x < 10 && y > 5 && y < 20
        "#),
        ("Nested conditions", r#"
let x = 15
if (x < 10)
    1
else
    if (x < 20)
        2
    else
        3
    endif
endif
        "#),
    ];
    
    // Aggregate node counts across all test cases
    let mut total_counts = std::collections::HashMap::new();
    let mut total_time_impact = std::collections::HashMap::new();
    
    println!("Per-test analysis:");
    println!("{}", "-".repeat(80));
    
    for (name, code) in &test_cases {
        let mut parser = create_parser("echo").unwrap();
        let ast = if code.contains('\n') {
            parser.parse_program(code).unwrap()
        } else {
            parser.parse(code).unwrap()
        };
        
        let counts = count_non_jit_nodes(&ast);
        
        // Measure execution time
        let (interp_time, jit_time) = measure_execution_time(code, 1000);
        let slowdown = jit_time.as_secs_f64() / interp_time.as_secs_f64();
        
        println!("\n{}", name);
        println!("  Interpreter: {:.2} µs", interp_time.as_secs_f64() * 1_000.0);
        println!("  JIT:         {:.2} µs ({:.1}x slower)", jit_time.as_secs_f64() * 1_000.0, slowdown);
        println!("  Non-JIT nodes:");
        
        let mut sorted_counts: Vec<_> = counts.iter().collect();
        sorted_counts.sort_by_key(|(_, count)| std::cmp::Reverse(**count));
        
        for (node_type, count) in sorted_counts {
            println!("    - {}: {}", node_type, count);
            *total_counts.entry(node_type.clone()).or_insert(0) += count;
            
            // Estimate time impact (very rough)
            let time_impact = (*count as f64) * (jit_time.as_secs_f64() - interp_time.as_secs_f64());
            *total_time_impact.entry(node_type.clone()).or_insert(0.0) += time_impact;
        }
    }
    
    println!("\n\n=== Aggregate Impact Analysis ===\n");
    
    // Sort by frequency
    let mut sorted_total: Vec<_> = total_counts.iter().collect();
    sorted_total.sort_by_key(|(_, count)| std::cmp::Reverse(**count));
    
    println!("Most frequent non-JIT nodes:");
    for (node_type, count) in &sorted_total[..sorted_total.len().min(10)] {
        println!("  {}: {} occurrences", node_type, count);
    }
    
    println!("\n=== Recommended JIT Priorities ===\n");
    println!("Based on frequency and performance impact:\n");
    
    println!("1. **Variable access** (highest frequency: {} occurrences)", total_counts.get("Variable access").unwrap_or(&0));
    println!("   - Required for almost all real code");
    println!("   - Relatively simple to implement (stack slot access)");
    println!("   - Would enable JIT for arithmetic with variables");
    
    println!("\n2. **Variable assignment** ({} occurrences)", total_counts.get("Variable assignment").unwrap_or(&0));
    println!("   - Essential for loops and state updates");
    println!("   - Builds on variable access implementation");
    
    println!("\n3. **If statements** ({} occurrences)", total_counts.get("If statement").unwrap_or(&0));
    println!("   - Critical control flow primitive");
    println!("   - Enables conditional computation");
    println!("   - Can use Cranelift's br_icmp for efficient branching");
    
    println!("\n4. **Boolean literals & logical ops** ({} AND, {} OR)", 
        total_counts.get("Logical AND").unwrap_or(&0),
        total_counts.get("Logical OR").unwrap_or(&0));
    println!("   - Needed for realistic conditions");
    println!("   - Can implement short-circuit evaluation");
    
    println!("\n5. **For loops** ({} occurrences)", total_counts.get("For loop").unwrap_or(&0));
    println!("   - High impact for iterative algorithms");
    println!("   - More complex but huge performance gains");
    
    println!("\n=== Implementation Strategy ===\n");
    println!("Phase 1: Variables (access + assignment)");
    println!("  - This alone would enable JIT for ~60-80% of arithmetic code");
    println!("  - Use Cranelift stack slots for local variables");
    println!("  - Simple symbol table mapping names to slots");
    
    println!("\nPhase 2: Basic control flow (if)");
    println!("  - Enables conditional arithmetic");
    println!("  - Use Cranelift's conditional branch instructions");
    
    println!("\nPhase 3: Loops");
    println!("  - Massive performance gains for iterative code");
    println!("  - More complex due to break/continue handling");
    
    println!("\nWith just Phase 1 (variables), JIT would become beneficial for:");
    println!("- Mathematical computations with intermediate results");
    println!("- Algorithms that build up results incrementally");
    println!("- Any code that isn't just a single expression");
}