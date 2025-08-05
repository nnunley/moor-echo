use std::fs;
use std::path::Path;
use std::collections::HashSet;
use echo_core::parser::moo_compat::MooCompatParser;
use echo_core::parser::Parser;

fn analyze_moo_file(path: &Path) -> Result<(usize, HashSet<String>), String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("File read error: {}", e))?;
    
    let lines: Vec<&str> = content.lines().collect();
    let mut features = HashSet::new();
    
    // Analyze language features used
    for line in &lines {
        let trimmed = line.trim();
        
        // Object definitions
        if trimmed.starts_with("object ") {
            features.insert("object_definition".to_string());
        }
        
        // Property definitions
        if trimmed.contains(" = ") && !trimmed.starts_with("verb") {
            features.insert("property_assignment".to_string());
        }
        
        // Verb definitions
        if trimmed.starts_with("verb ") {
            features.insert("verb_definition".to_string());
        }
        
        // Special MOO features
        if trimmed.contains("`") {
            features.insert("error_catching_expression".to_string());
        }
        
        if trimmed.contains("=>") {
            features.insert("arrow_operator".to_string());
        }
        
        if trimmed.contains("::") {
            features.insert("pass_statement".to_string());
        }
        
        if trimmed.starts_with("define ") {
            features.insert("define_statement".to_string());
        }
        
        if trimmed.contains("{") && trimmed.contains("}") && trimmed.contains("=") {
            features.insert("destructuring_assignment".to_string());
        }
        
        if trimmed.contains("...") {
            features.insert("spread_operator".to_string());
        }
        
        if trimmed.contains("$") && !trimmed.contains("\"") {
            features.insert("system_property".to_string());
        }
        
        if trimmed.contains("#") && (trimmed.contains("#-") || trimmed.contains("#[0-9]")) {
            features.insert("object_reference".to_string());
        }
        
        if trimmed.contains("@") {
            features.insert("list_splice".to_string());
        }
        
        if trimmed.contains("'") && !trimmed.contains("\"") {
            features.insert("symbol_literal".to_string());
        }
        
        if trimmed.contains("[") && trimmed.contains("->") && trimmed.contains("]") {
            features.insert("map_literal".to_string());
        }
        
        if trimmed.contains("try") && trimmed.contains("except") {
            features.insert("try_except".to_string());
        }
    }
    
    Ok((lines.len(), features))
}

fn main() {
    let moo_dir = "../cowbell/objdump-out/textdump-1743778867.moo";
    let moo_files = vec![
        "event.moo", "login.moo", "sysobj.moo", "sub.moo", "look.moo",
        "builder.moo", "room.moo", "wiz.moo", "thing.moo", "list.moo",
        "first_room.moo", "password.moo", "prog.moo", "constants.moo",
        "block.moo", "root.moo", "string.moo", "hacker.moo", "player.moo",
        "arch_wizard.moo",
    ];

    println!("Analyzing MOO language features used in cowbell...\n");

    let mut all_features = HashSet::new();
    let mut total_lines = 0;
    
    for file in &moo_files {
        let path = Path::new(moo_dir).join(file);
        
        match analyze_moo_file(&path) {
            Ok((lines, features)) => {
                println!("{}: {} lines, {} unique features", file, lines, features.len());
                if !features.is_empty() {
                    for feature in &features {
                        println!("  - {}", feature);
                        all_features.insert(feature.clone());
                    }
                }
                total_lines += lines;
            }
            Err(e) => {
                println!("{}: ERROR - {}", file, e);
            }
        }
        println!();
    }
    
    println!("\n=== SUMMARY ===");
    println!("Total lines of code: {}", total_lines);
    println!("Total unique features: {}", all_features.len());
    println!("\nAll features found:");
    let mut sorted_features: Vec<_> = all_features.iter().collect();
    sorted_features.sort();
    for feature in sorted_features {
        println!("  - {}", feature);
    }
    
    // Now test which files actually parse
    println!("\n=== PARSING TEST ===");
    let mut success_count = 0;
    let mut failure_count = 0;
    
    for file in &moo_files {
        let path = Path::new(moo_dir).join(file);
        if let Ok(content) = fs::read_to_string(&path) {
            let mut parser = MooCompatParser::new().unwrap();
            match parser.parse(&content) {
                Ok(_) => {
                    println!("{}: PARSE SUCCESS", file);
                    success_count += 1;
                }
                Err(e) => {
                    println!("{}: PARSE FAILED - {}", file, e);
                    failure_count += 1;
                }
            }
        }
    }
    
    println!("\nParsing success rate: {}/{} ({:.1}%)", 
             success_count, 
             moo_files.len(),
             (success_count as f64 / moo_files.len() as f64) * 100.0);
}