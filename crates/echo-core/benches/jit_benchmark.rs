use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use echo_core::{
    evaluator::Evaluator,
    parser::create_parser,
    storage::{Storage, ObjectId},
    ast::EchoAst,
};
#[cfg(feature = "jit")]
use echo_core::evaluator::JitEvaluator;
use std::sync::Arc;
use tempfile::TempDir;

enum EvaluatorType {
    Interpreted(Evaluator),
    #[cfg(feature = "jit")]
    Jit(JitEvaluator),
}

impl EvaluatorType {
    fn eval_with_player(&mut self, ast: &echo_core::ast::EchoAst, player_id: ObjectId) -> anyhow::Result<echo_core::Value> {
        match self {
            EvaluatorType::Interpreted(e) => e.eval_with_player(ast, player_id),
            #[cfg(feature = "jit")]
            EvaluatorType::Jit(e) => e.eval_with_player(ast, player_id),
        }
    }
    
    fn create_player(&mut self, name: &str) -> anyhow::Result<ObjectId> {
        match self {
            EvaluatorType::Interpreted(e) => e.create_player(name),
            #[cfg(feature = "jit")]
            EvaluatorType::Jit(e) => e.create_player(name),
        }
    }
}

fn setup_evaluator(use_jit: bool) -> (EvaluatorType, ObjectId, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let storage = Arc::new(Storage::new(temp_dir.path()).unwrap());
    
    let mut evaluator = if use_jit {
        #[cfg(feature = "jit")]
        {
            EvaluatorType::Jit(JitEvaluator::new_with_fallback(storage))
        }
        #[cfg(not(feature = "jit"))]
        {
            println!("Warning: JIT feature not enabled, using interpreter");
            EvaluatorType::Interpreted(Evaluator::new(storage))
        }
    } else {
        EvaluatorType::Interpreted(Evaluator::new(storage))
    };
    
    let player_id = evaluator.create_player("bench").unwrap();
    (evaluator, player_id, temp_dir)
}

/// Analyze an AST to determine what percentage can be JIT compiled
fn analyze_jit_coverage(ast: &EchoAst) -> (usize, usize, Vec<String>) {
    let mut total_nodes = 0;
    let mut jittable_nodes = 0;
    let mut non_jittable_reasons = Vec::new();
    
    fn visit(ast: &EchoAst, total: &mut usize, jittable: &mut usize, reasons: &mut Vec<String>) {
        *total += 1;
        
        match ast {
            // JIT-compilable nodes
            EchoAst::Number(_) => *jittable += 1,
            EchoAst::Add { left, right } |
            EchoAst::Subtract { left, right } |
            EchoAst::Multiply { left, right } |
            EchoAst::Divide { left, right } |
            EchoAst::Modulo { left, right } => {
                *jittable += 1;
                visit(left, total, jittable, reasons);
                visit(right, total, jittable, reasons);
            }
            EchoAst::Equal { left, right } |
            EchoAst::NotEqual { left, right } |
            EchoAst::LessThan { left, right } |
            EchoAst::LessEqual { left, right } |
            EchoAst::GreaterThan { left, right } |
            EchoAst::GreaterEqual { left, right } => {
                *jittable += 1;
                visit(left, total, jittable, reasons);
                visit(right, total, jittable, reasons);
            }
            EchoAst::UnaryMinus { operand } => {
                *jittable += 1;
                visit(operand, total, jittable, reasons);
            }
            
            // Non-JIT nodes
            EchoAst::String(_) => reasons.push("String literal".to_string()),
            EchoAst::Boolean(_) => reasons.push("Boolean literal".to_string()),
            EchoAst::Identifier(_) => reasons.push("Variable access".to_string()),
            EchoAst::For { .. } => reasons.push("For loop".to_string()),
            EchoAst::While { .. } => reasons.push("While loop".to_string()),
            EchoAst::If { .. } => reasons.push("If statement".to_string()),
            EchoAst::LocalAssignment { .. } => reasons.push("Variable assignment".to_string()),
            EchoAst::List { .. } => reasons.push("List literal".to_string()),
            EchoAst::Match { .. } => reasons.push("Match expression".to_string()),
            EchoAst::SystemProperty(_) => reasons.push("System property access".to_string()),
            EchoAst::FunctionCall { .. } => reasons.push("Function call".to_string()),
            EchoAst::Power { .. } => reasons.push("Power operation".to_string()),
            
            // Recurse into compound expressions
            EchoAst::Program(stmts) | 
            EchoAst::Block(stmts) => {
                for stmt in stmts {
                    visit(stmt, total, jittable, reasons);
                }
            }
            
            _ => reasons.push(format!("Unsupported AST node")),
        }
    }
    
    visit(ast, &mut total_nodes, &mut jittable_nodes, &mut non_jittable_reasons);
    (total_nodes, jittable_nodes, non_jittable_reasons)
}

fn bench_arithmetic(c: &mut Criterion) {
    let mut group = c.benchmark_group("arithmetic");
    
    // Simple arithmetic expression
    let code = "1 + 2 * 3 - 4 / 2";
    let mut parser = create_parser("echo").unwrap();
    let ast = parser.parse(code).unwrap();
    
    group.bench_function("interpreted", |b| {
        let (mut evaluator, player_id, _temp_dir) = setup_evaluator(false);
        b.iter(|| {
            let result = evaluator.eval_with_player(&ast, player_id).unwrap();
            black_box(result);
        });
    });
    
    group.bench_function("jit", |b| {
        let (mut evaluator, player_id, _temp_dir) = setup_evaluator(true);
        // Warm up JIT
        evaluator.eval_with_player(&ast, player_id).unwrap();
        b.iter(|| {
            let result = evaluator.eval_with_player(&ast, player_id).unwrap();
            black_box(result);
        });
    });
    
    group.finish();
}

fn bench_loop(c: &mut Criterion) {
    let mut group = c.benchmark_group("loop");
    
    // Loop that sums numbers
    let code = r#"
let sum = 0
for i in ([1, 2, 3, 4, 5, 6, 7, 8, 9, 10])
    sum = sum + i
endfor
sum
    "#;
    
    let mut parser = create_parser("echo").unwrap();
    let ast = parser.parse_program(code).unwrap();
    
    group.bench_function("interpreted", |b| {
        let (mut evaluator, player_id, _temp_dir) = setup_evaluator(false);
        b.iter(|| {
            let result = evaluator.eval_with_player(&ast, player_id).unwrap();
            black_box(result);
        });
    });
    
    group.bench_function("jit", |b| {
        let (mut evaluator, player_id, _temp_dir) = setup_evaluator(true);
        // Warm up JIT
        evaluator.eval_with_player(&ast, player_id).unwrap();
        b.iter(|| {
            let result = evaluator.eval_with_player(&ast, player_id).unwrap();
            black_box(result);
        });
    });
    
    group.finish();
}

fn bench_factorial(c: &mut Criterion) {
    let mut group = c.benchmark_group("factorial");
    
    // Iterative factorial calculation
    for n in [5, 10, 15].iter() {
        let fact_code = format!(r#"
let result = 1
for i in ([{}])
    result = result * i
endfor
result
        "#, (1..=*n).map(|i| i.to_string()).collect::<Vec<_>>().join(", "));
        
        let mut parser = create_parser("echo").unwrap();
        let ast = parser.parse_program(&fact_code).unwrap();
        
        group.bench_with_input(BenchmarkId::new("interpreted", n), n, |b, _| {
            let (mut evaluator, player_id, _temp_dir) = setup_evaluator(false);
            b.iter(|| {
                let result = evaluator.eval_with_player(&ast, player_id).unwrap();
                black_box(result);
            });
        });
        
        group.bench_with_input(BenchmarkId::new("jit", n), n, |b, _| {
            let (mut evaluator, player_id, _temp_dir) = setup_evaluator(true);
            // Warm up JIT
            evaluator.eval_with_player(&ast, player_id).unwrap();
            b.iter(|| {
                let result = evaluator.eval_with_player(&ast, player_id).unwrap();
                black_box(result);
            });
        });
    }
    
    group.finish();
}

fn bench_system_property(c: &mut Criterion) {
    let mut group = c.benchmark_group("system_property");
    
    // Test system property access by accessing the player registry
    // which is automatically created when we create a player
    let code = "$player_registry";
    
    let mut parser = create_parser("echo").unwrap();
    let ast = parser.parse(code).unwrap();
    
    group.bench_function("interpreted", |b| {
        let (mut evaluator, player_id, _temp_dir) = setup_evaluator(false);
        b.iter(|| {
            let result = evaluator.eval_with_player(&ast, player_id).unwrap();
            black_box(result);
        });
    });
    
    group.bench_function("jit", |b| {
        let (mut evaluator, player_id, _temp_dir) = setup_evaluator(true);
        // Warm up JIT
        evaluator.eval_with_player(&ast, player_id).unwrap();
        b.iter(|| {
            let result = evaluator.eval_with_player(&ast, player_id).unwrap();
            black_box(result);
        });
    });
    
    group.finish();
}

fn bench_list_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("list_operations");
    
    // Test list operations
    let code = r#"
let list = [1, 2, 3, 4, 5]
let sum = 0
for item in (list)
    sum = sum + item
endfor
sum
    "#;
    
    let mut parser = create_parser("echo").unwrap();
    let ast = parser.parse_program(code).unwrap();
    
    group.bench_function("interpreted", |b| {
        let (mut evaluator, player_id, _temp_dir) = setup_evaluator(false);
        b.iter(|| {
            let result = evaluator.eval_with_player(&ast, player_id).unwrap();
            black_box(result);
        });
    });
    
    group.bench_function("jit", |b| {
        let (mut evaluator, player_id, _temp_dir) = setup_evaluator(true);
        // Warm up JIT
        evaluator.eval_with_player(&ast, player_id).unwrap();
        b.iter(|| {
            let result = evaluator.eval_with_player(&ast, player_id).unwrap();
            black_box(result);
        });
    });
    
    group.finish();
}

fn bench_string_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("string_operations");
    
    // String concatenation
    let code = r#""Hello, " + "world" + "!" + " from " + "Echo""#;
    
    let mut parser = create_parser("echo").unwrap();
    let ast = parser.parse(code).unwrap();
    
    group.bench_function("interpreted", |b| {
        let (mut evaluator, player_id, _temp_dir) = setup_evaluator(false);
        b.iter(|| {
            let result = evaluator.eval_with_player(&ast, player_id).unwrap();
            black_box(result);
        });
    });
    
    group.bench_function("jit", |b| {
        let (mut evaluator, player_id, _temp_dir) = setup_evaluator(true);
        // Warm up JIT
        evaluator.eval_with_player(&ast, player_id).unwrap();
        b.iter(|| {
            let result = evaluator.eval_with_player(&ast, player_id).unwrap();
            black_box(result);
        });
    });
    
    group.finish();
}

fn bench_match_expression(c: &mut Criterion) {
    let mut group = c.benchmark_group("match_expression");
    
    // Match expression
    let code = r#"
match 42
case 1 => "one"
case 2 => "two"
case 42 => "forty-two"
case _ => "other"
endmatch
    "#;
    
    let mut parser = create_parser("echo").unwrap();
    let ast = parser.parse(code).unwrap();
    
    group.bench_function("interpreted", |b| {
        let (mut evaluator, player_id, _temp_dir) = setup_evaluator(false);
        b.iter(|| {
            let result = evaluator.eval_with_player(&ast, player_id).unwrap();
            black_box(result);
        });
    });
    
    group.bench_function("jit", |b| {
        let (mut evaluator, player_id, _temp_dir) = setup_evaluator(true);
        // Warm up JIT
        evaluator.eval_with_player(&ast, player_id).unwrap();
        b.iter(|| {
            let result = evaluator.eval_with_player(&ast, player_id).unwrap();
            black_box(result);
        });
    });
    
    group.finish();
}

fn bench_mixed_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("mixed_operations");
    
    // Complex expression mixing different operations
    let code = r#"
let x = 10
let y = 20
let result = (x + y) * 2 - (x / 2) + (y % 3)
if (result > 50)
    "high"
else
    "low"
endif
    "#;
    
    let mut parser = create_parser("echo").unwrap();
    let ast = parser.parse_program(code).unwrap();
    
    group.bench_function("interpreted", |b| {
        let (mut evaluator, player_id, _temp_dir) = setup_evaluator(false);
        b.iter(|| {
            let result = evaluator.eval_with_player(&ast, player_id).unwrap();
            black_box(result);
        });
    });
    
    group.bench_function("jit", |b| {
        let (mut evaluator, player_id, _temp_dir) = setup_evaluator(true);
        // Warm up JIT
        evaluator.eval_with_player(&ast, player_id).unwrap();
        b.iter(|| {
            let result = evaluator.eval_with_player(&ast, player_id).unwrap();
            black_box(result);
        });
    });
    
    group.finish();
}

fn analyze_jit_coverage_report() {
    println!("\n=== JIT Coverage Analysis ===\n");
    
    let test_cases = vec![
        ("Arithmetic", "1 + 2 * 3 - 4 / 2"),
        ("Variable arithmetic", "let x = 10\nx + 5"),
        ("Pure arithmetic tree", "((1 + 2) * (3 - 4)) / (5 + 6)"),
        ("Loop", r#"
let sum = 0
for i in ([1, 2, 3, 4, 5])
    sum = sum + i
endfor
sum
        "#),
        ("Comparison chain", "1 < 2 && 3 > 2 && 4 == 4"),
        ("If statement", "if (1 < 2) 3 else 4 endif"),
        ("Function call", "max(1, 2)"),
        ("String concat", r#""Hello" + " " + "World""#),
    ];
    
    let mut parser = create_parser("echo").unwrap();
    
    for (name, code) in test_cases {
        let ast = if code.contains('\n') {
            parser.parse_program(code).unwrap()
        } else {
            parser.parse(code).unwrap()
        };
        
        let (total, jittable, reasons) = analyze_jit_coverage(&ast);
        let percentage = if total > 0 { (jittable as f64 / total as f64) * 100.0 } else { 0.0 };
        
        println!("{}: {:.1}% JIT coverage ({}/{} nodes)", name, percentage, jittable, total);
        if !reasons.is_empty() {
            let mut reason_counts = std::collections::HashMap::new();
            for reason in &reasons {
                *reason_counts.entry(reason.as_str()).or_insert(0) += 1;
            }
            print!("  Non-JIT: ");
            for (i, (reason, count)) in reason_counts.iter().enumerate() {
                if i > 0 { print!(", "); }
                print!("{} ({}x)", reason, count);
            }
            println!();
        }
        println!();
    }
}

criterion_group!(
    benches,
    bench_arithmetic,
    bench_loop,
    bench_factorial,
    bench_system_property,
    bench_list_operations,
    bench_string_operations,
    bench_match_expression,
    bench_mixed_operations
);
criterion_main!(benches);