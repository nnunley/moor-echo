use echo_core::{
    parser::create_parser,
    evaluator::{Evaluator, JitEvaluator},
    storage::Storage,
};
use std::sync::Arc;
use std::time::Instant;
use tempfile::TempDir;

#[derive(Debug)]
struct BenchmarkResult {
    name: String,
    interp_time_us: f64,
    jit_cold_time_us: f64,
    jit_warm_time_us: f64,
    compilation_overhead: f64,
    warm_overhead: f64,
    category: String,
}

impl BenchmarkResult {
    fn compilation_slowdown(&self) -> f64 {
        self.jit_cold_time_us / self.interp_time_us
    }
    
    fn warm_slowdown(&self) -> f64 {
        self.jit_warm_time_us / self.interp_time_us
    }
    
    fn optimization_potential(&self) -> f64 {
        // Higher values = more potential for optimization
        // Factors: compilation overhead, warm overhead, expression complexity
        let comp_factor = (self.compilation_slowdown() - 1.0).max(0.0);
        let warm_factor = (self.warm_slowdown() - 1.0).max(0.0);
        let complexity_factor = self.interp_time_us / 10.0; // Normalize by 10µs
        
        comp_factor * 0.5 + warm_factor * 0.3 + complexity_factor * 0.2
    }
}

fn benchmark_expression(name: &str, code: &str, category: &str, iterations: usize) -> BenchmarkResult {
    let temp_dir = TempDir::new().unwrap();
    let storage = Arc::new(Storage::new(temp_dir.path()).unwrap());
    
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
    let interp_time = start.elapsed().as_secs_f64() * 1_000_000.0 / iterations as f64;
    
    // Benchmark JIT cold start
    let mut jit = JitEvaluator::new_with_fallback(storage.clone());
    let player_id = jit.create_player("bench").unwrap();
    
    let start = Instant::now();
    for _ in 0..iterations {
        jit.eval_with_player(&ast, player_id).unwrap();
    }
    let jit_cold_time = start.elapsed().as_secs_f64() * 1_000_000.0 / iterations as f64;
    
    // Benchmark JIT warm
    let mut jit_warm = JitEvaluator::new_with_fallback(storage);
    let player_id = jit_warm.create_player("bench").unwrap();
    
    // Warm up
    jit_warm.eval_with_player(&ast, player_id).unwrap();
    
    let start = Instant::now();
    for _ in 0..iterations {
        jit_warm.eval_with_player(&ast, player_id).unwrap();
    }
    let jit_warm_time = start.elapsed().as_secs_f64() * 1_000_000.0 / iterations as f64;
    
    BenchmarkResult {
        name: name.to_string(),
        interp_time_us: interp_time,
        jit_cold_time_us: jit_cold_time,
        jit_warm_time_us: jit_warm_time,
        compilation_overhead: jit_cold_time - jit_warm_time,
        warm_overhead: jit_warm_time - interp_time,
        category: category.to_string(),
    }
}

fn main() {
    println!("🔬 JIT OPTIMIZATION ANALYSIS");
    println!("===============================\n");
    
    let mut results = Vec::new();
    let iterations = 5000;
    
    // Test different categories of expressions
    let test_cases = vec![
        // Simple expressions (should be fastest wins)
        ("Simple Add", "1 + 2", "arithmetic"),
        ("Simple Boolean", "true", "boolean"),
        ("Simple Compare", "5 < 10", "comparison"),
        
        // Medium complexity
        ("Medium Arithmetic", "1 + 2 * 3 - 4 / 2", "arithmetic"),
        ("Medium Boolean", "true && false || true", "boolean"),
        ("Medium Compare", "5 < 10 && 15 > 12", "comparison"),
        
        // Complex expressions
        ("Complex Arithmetic", "((1 + 2) * (3 - 4)) / (5 + 6)", "arithmetic"),
        ("Complex Boolean", "true && false || true && false || true", "boolean"),
        ("Complex Compare", "1 < 2 && 3 > 2 && 4 == 4 && 5 <= 6", "comparison"),
        
        // Mixed operations
        ("Mixed Simple", "(5 + 3) > 7", "mixed"),
        ("Mixed Medium", "true && (5 + 3) > 7", "mixed"),
        ("Mixed Complex", "(5 + 3) * 2 > 10 && (20 / 4) == 5", "mixed"),
        
        // Control flow
        ("If True", "if (true) 42 else 0 endif", "control"),
        ("If Compare", "if (5 < 10) 100 else 200 endif", "control"),
    ];
    
    for (name, code, category) in test_cases {
        let result = benchmark_expression(name, code, category, iterations);
        println!("{:20} | {:8.2} µs | {:8.2} µs | {:8.2} µs | {:6.1}x | {:6.1}x", 
            result.name,
            result.interp_time_us,
            result.jit_cold_time_us,
            result.jit_warm_time_us,
            result.compilation_slowdown(),
            result.warm_slowdown()
        );
        results.push(result);
    }
    
    println!("\n📊 OPTIMIZATION ANALYSIS");
    println!("=========================\n");
    
    // Sort by optimization potential
    results.sort_by(|a, b| b.optimization_potential().partial_cmp(&a.optimization_potential()).unwrap());
    
    println!("🎯 TOP OPTIMIZATION TARGETS (by potential impact):");
    for (i, result) in results.iter().take(5).enumerate() {
        println!("{}. {} ({})", 
            i + 1, 
            result.name, 
            result.category
        );
        println!("   Compilation overhead: {:.1}µs ({:.1}x slower)", 
            result.compilation_overhead, 
            result.compilation_slowdown()
        );
        println!("   Warm overhead: {:.1}µs ({:.1}x slower)", 
            result.warm_overhead, 
            result.warm_slowdown()
        );
        println!("   Optimization potential: {:.2}", result.optimization_potential());
        println!();
    }
    
    // Analyze by category
    println!("📈 CATEGORY ANALYSIS:");
    let categories = ["arithmetic", "boolean", "comparison", "mixed", "control"];
    for category in categories {
        let cat_results: Vec<_> = results.iter().filter(|r| r.category == category).collect();
        if cat_results.is_empty() { continue; }
        
        let avg_comp_slowdown: f64 = cat_results.iter().map(|r| r.compilation_slowdown()).sum::<f64>() / cat_results.len() as f64;
        let avg_warm_slowdown: f64 = cat_results.iter().map(|r| r.warm_slowdown()).sum::<f64>() / cat_results.len() as f64;
        
        println!("{:12} | Avg Cold: {:4.1}x | Avg Warm: {:4.1}x | Count: {}", 
            category.to_uppercase(), 
            avg_comp_slowdown, 
            avg_warm_slowdown, 
            cat_results.len()
        );
    }
    
    println!("\n🚀 LOW HANGING FRUIT RECOMMENDATIONS:");
    println!("=====================================");
    
    println!("\n1. **COMPILATION THRESHOLD** 🎯");
    println!("   Current issue: Even simple expressions take 15-130µs to compile");
    println!("   Solution: Only JIT expressions that take >X µs to interpret");
    let threshold_candidates: Vec<_> = results.iter()
        .filter(|r| r.interp_time_us > 1.0)
        .collect();
    if !threshold_candidates.is_empty() {
        let min_beneficial = threshold_candidates.iter()
            .map(|r| r.interp_time_us)
            .fold(f64::INFINITY, f64::min);
        println!("   Recommended threshold: {:.1}µs interpreter time", min_beneficial);
    }
    
    println!("\n2. **CRANELIFT OPTIMIZATION** ⚡");
    println!("   Current issue: High compilation overhead for all expressions");
    println!("   Solutions:");
    println!("   - Use OptLevel::Speed instead of default");
    println!("   - Enable function caching/memoization");
    println!("   - Use simpler Cranelift settings for small expressions");
    
    println!("\n3. **EXPRESSION COMPLEXITY ANALYSIS** 🧮");
    let simple_expressions: Vec<_> = results.iter()
        .filter(|r| r.interp_time_us < 0.5)
        .collect();
    println!("   {} expressions take <0.5µs to interpret", simple_expressions.len());
    println!("   These should NEVER be JIT compiled");
    
    let medium_expressions: Vec<_> = results.iter()
        .filter(|r| r.interp_time_us >= 0.5 && r.interp_time_us < 2.0)
        .collect();
    println!("   {} expressions take 0.5-2.0µs to interpret", medium_expressions.len());
    println!("   These need optimized JIT compilation");
    
    let complex_expressions: Vec<_> = results.iter()
        .filter(|r| r.interp_time_us >= 2.0)
        .collect();
    println!("   {} expressions take >2.0µs to interpret", complex_expressions.len());
    println!("   These are good JIT candidates");
    
    println!("\n4. **IMMEDIATE WINS** 🏆");
    println!("   Priority 1: Add interpreter time threshold (5min fix)");
    println!("   Priority 2: Cache compiled functions by AST hash (2hr fix)");
    println!("   Priority 3: Optimize Cranelift settings (1hr fix)");
    println!("   Priority 4: Pre-compile common patterns (4hr fix)");
    
    println!("\n5. **PERFORMANCE TARGETS** 🎯");
    println!("   Target: JIT warm time ≤ 2x interpreter time");
    println!("   Target: JIT cold time ≤ 10x interpreter time for complex expressions");
    println!("   Target: JIT compilation time ≤ 50µs for simple expressions");
    
    let potential_wins: Vec<_> = results.iter()
        .filter(|r| r.interp_time_us > 1.0 && r.warm_slowdown() < 5.0)
        .collect();
    println!("\n   {} expressions show JIT potential with optimization", potential_wins.len());
    for result in potential_wins.iter().take(3) {
        println!("   - {}: {:.1}µs → target {:.1}µs JIT", 
            result.name, 
            result.interp_time_us, 
            result.interp_time_us * 2.0
        );
    }
}