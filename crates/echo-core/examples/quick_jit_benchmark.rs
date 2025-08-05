use echo_core::{
    parser::create_parser,
    evaluator::{Evaluator, JitEvaluator},
    storage::Storage,
};
use std::sync::Arc;
use std::time::Instant;
use tempfile::TempDir;

fn benchmark_expression(name: &str, code: &str, iterations: usize) -> (f64, f64, f64) {
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
    let interp_time = start.elapsed().as_secs_f64();
    
    // Benchmark JIT
    let mut jit = JitEvaluator::new_with_fallback(storage);
    let player_id = jit.create_player("bench").unwrap();
    
    // Cold start (include compilation time)
    let start = Instant::now();
    for _ in 0..iterations {
        jit.eval_with_player(&ast, player_id).unwrap();
    }
    let jit_cold_time = start.elapsed().as_secs_f64();
    
    // Warm start (compilation cached)
    let start = Instant::now();
    for _ in 0..iterations {
        jit.eval_with_player(&ast, player_id).unwrap();
    }
    let jit_warm_time = start.elapsed().as_secs_f64();
    
    let cold_speedup = interp_time / jit_cold_time;
    let warm_speedup = interp_time / jit_warm_time;
    
    println!("{:30} | {:8.2} µs | {:8.2} µs | {:8.2} µs | {:6.2}x | {:6.2}x", 
        name,
        interp_time * 1_000_000.0 / iterations as f64,
        jit_cold_time * 1_000_000.0 / iterations as f64,
        jit_warm_time * 1_000_000.0 / iterations as f64,
        cold_speedup,
        warm_speedup
    );
    
    (interp_time, jit_cold_time, jit_warm_time)
}

fn main() {
    println!("JIT Performance Analysis - Low Hanging Fruit");
    println!("==============================================\n");
    
    println!("{:30} | {:>10} | {:>10} | {:>10} | {:>8} | {:>8}", 
        "Expression", "Interp", "JIT Cold", "JIT Warm", "Cold", "Warm");
    println!("{}", "-".repeat(85));
    
    let iterations = 10000;
    
    // Category 1: Pure arithmetic (should be fastest JIT wins)
    println!("\n🧮 PURE ARITHMETIC (Expected JIT wins):");
    benchmark_expression("Simple Add", "1 + 2", iterations);
    benchmark_expression("Complex Arithmetic", "1 + 2 * 3 - 4 / 2", iterations);
    benchmark_expression("Deep Arithmetic", "((1 + 2) * (3 - 4)) / (5 + 6)", iterations);
    benchmark_expression("Many Operations", "1 + 2 + 3 + 4 + 5 + 6 + 7 + 8", iterations);
    
    // Category 2: Boolean operations
    println!("\n🔤 BOOLEAN OPERATIONS:");
    benchmark_expression("Simple Boolean", "true", iterations);
    benchmark_expression("Boolean AND", "true && false", iterations);
    benchmark_expression("Boolean OR", "false || true", iterations);
    benchmark_expression("Complex Boolean", "true && false || true && false", iterations);
    
    // Category 3: Comparisons
    println!("\n⚖️ COMPARISONS:");
    benchmark_expression("Simple Compare", "5 < 10", iterations);
    benchmark_expression("Chained Compare", "1 < 2 && 3 > 2", iterations);
    benchmark_expression("Mixed Compare", "5 < 10 && 15 > 12 && 7 == 7", iterations);
    
    // Category 4: Mixed operations (JIT-compilable)
    println!("\n🔀 MIXED OPERATIONS:");
    benchmark_expression("Arith + Compare", "(5 + 3) > 7", iterations);
    benchmark_expression("Bool + Arith", "true && (5 + 3) > 7", iterations);
    benchmark_expression("Complex Mixed", "(5 + 3) * 2 > 10 && (20 / 4) == 5", iterations);
    
    // Category 5: Variables (fallback to interpreter)
    println!("\n📦 VARIABLES (Expected interpreter fallback):");
    let var_iterations = 1000; // Fewer iterations for fallback cases
    benchmark_expression("Simple Variable", "let x = 5\nx", var_iterations);
    benchmark_expression("Variable Arithmetic", "let x = 5\nx + 10", var_iterations);
    
    // Category 6: Control flow (partially working)
    println!("\n🎛️ CONTROL FLOW:");
    benchmark_expression("Simple If True", "if (true) 42 else 0 endif", var_iterations);
    benchmark_expression("Simple If False", "if (false) 42 else 99 endif", var_iterations);
    benchmark_expression("If with Compare", "if (5 < 10) 100 else 200 endif", var_iterations);
    
    println!("\n");
    println!("🎯 LOW HANGING FRUIT ANALYSIS:");
    println!("================================");
    println!("1. **Compilation Overhead**: JIT cold start is much slower");
    println!("2. **Simple Expressions**: Even basic arithmetic shows overhead");
    println!("3. **Boolean Operations**: Should be very fast but show high overhead");
    println!("4. **Threshold Analysis**: Need to find complexity threshold for JIT benefit");
    println!("5. **Caching**: JIT warm performance is better but still slower than interpreter");
    
    println!("\n🚀 OPTIMIZATION OPPORTUNITIES:");
    println!("1. **Reduce compilation overhead** for small expressions");
    println!("2. **Better caching** of compiled functions");
    println!("3. **Inline simple operations** without full JIT overhead");
    println!("4. **Compile-time detection** of JIT-beneficial expressions");
    println!("5. **Optimize Cranelift settings** for faster compilation");
}