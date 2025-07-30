use echo_core::{
    parser::create_parser,
    ast::EchoAst,
};
use std::collections::HashMap;

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
            EchoAst::For { collection, body, .. } => {
                reasons.push("For loop".to_string());
                visit(collection.as_ref(), total, jittable, reasons);
                for stmt in body {
                    visit(stmt, total, jittable, reasons);
                }
            }
            EchoAst::While { condition, body, .. } => {
                reasons.push("While loop".to_string());
                visit(condition, total, jittable, reasons);
                for stmt in body {
                    visit(stmt, total, jittable, reasons);
                }
            }
            EchoAst::If { condition, then_branch, else_branch, .. } => {
                reasons.push("If statement".to_string());
                visit(condition, total, jittable, reasons);
                for stmt in then_branch {
                    visit(stmt, total, jittable, reasons);
                }
                if let Some(else_body) = else_branch {
                    for stmt in else_body {
                        visit(stmt, total, jittable, reasons);
                    }
                }
            }
            EchoAst::LocalAssignment { value, .. } => {
                reasons.push("Variable assignment".to_string());
                visit(value, total, jittable, reasons);
            }
            EchoAst::List { elements, .. } => {
                reasons.push("List literal".to_string());
                for elem in elements {
                    visit(elem, total, jittable, reasons);
                }
            }
            EchoAst::Match { expr, arms, .. } => {
                reasons.push("Match expression".to_string());
                visit(expr, total, jittable, reasons);
                for arm in arms {
                    visit(&arm.body, total, jittable, reasons);
                    if arm.guard.is_some() {
                        reasons.push("Match guard".to_string());
                    }
                }
            }
            EchoAst::SystemProperty(_) => reasons.push("System property access".to_string()),
            EchoAst::FunctionCall { name, args, .. } => {
                reasons.push(format!("Function call: {}", name).to_string());
                for arg in args {
                    visit(arg, total, jittable, reasons);
                }
            }
            EchoAst::Power { left, right } => {
                reasons.push("Power operation".to_string());
                visit(left, total, jittable, reasons);
                visit(right, total, jittable, reasons);
            }
            EchoAst::And { left, right } => {
                reasons.push("Logical AND".to_string());
                visit(left, total, jittable, reasons);
                visit(right, total, jittable, reasons);
            }
            EchoAst::Or { left, right } => {
                reasons.push("Logical OR".to_string());
                visit(left, total, jittable, reasons);
                visit(right, total, jittable, reasons);
            }
            
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

fn main() {
    println!("\n=== JIT Coverage Analysis ===\n");
    
    let test_cases = vec![
        ("Arithmetic", "1 + 2 * 3 - 4 / 2"),
        ("Variable arithmetic", "let x = 10\nx + 5"),
        ("Pure arithmetic tree", "((1 + 2) * (3 - 4)) / (5 + 6)"),
        ("Nested arithmetic", "(1 + 2) * (3 + 4) - (5 * 6) / (7 - 8)"),
        ("Deep arithmetic", "1 + 2 + 3 + 4 + 5 + 6 + 7 + 8 + 9 + 10"),
        ("Loop", r#"
let sum = 0
for i in ([1, 2, 3, 4, 5])
    sum = sum + i
endfor
sum
        "#),
        ("Comparison", "1 < 2"),
        ("Comparison chain", "1 < 2 && 3 > 2 && 4 == 4"),
        ("Pure comparison", "1 < 2 == 3 > 2"),
        ("If statement", "if (1 < 2) 3 else 4 endif"),
        ("Function call", "max(1, 2)"),
        ("String concat", r#""Hello" + " " + "World""#),
        ("Mixed operations", r#"
let x = 10
let y = 20
if (x < y)
    x + y
else
    x - y
endif
        "#),
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
            let mut reason_counts = HashMap::new();
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
    
    println!("\n=== Summary ===");
    println!("Current JIT implementation can only compile:");
    println!("- Basic arithmetic: +, -, *, /, %");
    println!("- Basic comparisons: ==, !=, <, <=, >, >=");
    println!("- Unary minus");
    println!("\nEverything else falls back to interpreter, including:");
    println!("- Variable access/assignment");
    println!("- Control flow (if, while, for)");
    println!("- Function calls");
    println!("- String operations");
    println!("- Boolean operations (&&, ||)");
    println!("- Lists and property access");
}