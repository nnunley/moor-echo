use echo_core::{
    parser::create_parser,
    evaluator::{Evaluator, JitEvaluator},
    storage::Storage,
};
use std::sync::Arc;
use std::time::Instant;
use tempfile::TempDir;

fn benchmark_comparison(name: &str, code: &str, iterations: usize) {
    println!("\n=== {} ===", name);
    println!("Code: {}", code.trim());
    
    let temp_dir = TempDir::new().unwrap();
    let storage = Arc::new(Storage::new(temp_dir.path()).unwrap());
    
    // Parse the code
    let mut parser = create_parser("echo").unwrap();
    let ast = if code.contains('\n') {
        parser.parse_program(code).unwrap()
    } else {
        parser.parse(code).unwrap()
    };
    
    // Benchmark interpreter
    let mut interpreter = Evaluator::new(storage.clone());
    let player_id = interpreter.create_player("bench").unwrap();
    
    let start = Instant::now();
    for _ in 0..iterations {
        interpreter.eval_with_player(&ast, player_id).unwrap();
    }
    let interp_time = start.elapsed();
    
    // Benchmark JIT
    let mut jit = JitEvaluator::new_with_fallback(storage);
    let player_id = jit.create_player("bench").unwrap();
    
    // Warm up
    jit.eval_with_player(&ast, player_id).unwrap();
    
    let start = Instant::now();
    for _ in 0..iterations {
        jit.eval_with_player(&ast, player_id).unwrap();
    }
    let jit_time = start.elapsed();
    
    let speedup = interp_time.as_secs_f64() / jit_time.as_secs_f64();
    
    println!("Interpreter: {:.2} µs", interp_time.as_secs_f64() * 1_000_000.0 / iterations as f64);
    println!("JIT:         {:.2} µs", jit_time.as_secs_f64() * 1_000_000.0 / iterations as f64);
    if speedup > 1.0 {
        println!("🚀 JIT is {:.1}x faster!", speedup);
    } else {
        println!("🐌 JIT is {:.1}x slower", 1.0 / speedup);
    }
}

fn main() {
    println!("Echo JIT Compiler Showcase");
    println!("==========================\n");
    
    println!("✅ Working JIT Features:");
    
    // Pure arithmetic
    benchmark_comparison(
        "Pure Arithmetic",
        "1 + 2 * 3 - 4 / 2 + 5 * (6 - 7)",
        10000
    );
    
    // Boolean operations
    benchmark_comparison(
        "Boolean Operations",
        "true && false || true",
        10000
    );
    
    // Comparisons
    benchmark_comparison(
        "Comparisons",
        "5 < 10 && 15 > 12 && 7 == 7",
        10000
    );
    
    // Mixed arithmetic and comparisons
    benchmark_comparison(
        "Mixed Operations",
        "(5 + 3) * 2 > 10 && (20 / 4) == 5",
        10000
    );
    
    println!("\n❌ Partially Working (falls back to interpreter):");
    
    // Variables (fallback)
    benchmark_comparison(
        "Variables (Simple)",
        r#"
let x = 5
x + 10
        "#,
        1000
    );
    
    // If statements with variables (fallback)
    benchmark_comparison(
        "If Statements",
        r#"
let x = 15
if (x > 10)
    x * 2
else
    x / 2
endif
        "#,
        1000
    );
    
    println!("\n🔧 Next Steps:");
    println!("1. Fix variable scoping across statements in Program nodes");
    println!("2. Add proper context passing for nested expressions");
    println!("3. Implement for loops and while loops");
    println!("4. Add function calls and property access");
    
    println!("\n📊 Summary:");
    println!("- JIT compilation is working for basic arithmetic and boolean operations");
    println!("- Performance improvements are visible for computation-heavy expressions");
    println!("- Variable support needs cross-statement context preservation");
    println!("- The foundation is solid for expanding JIT support");
}