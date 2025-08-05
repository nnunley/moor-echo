use std::fs;
use pest::Parser;
use echo_core::parser::lambdamoo_db_parser::{LambdaMooDbParser, Rule};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Debugging LambdaCore verb section where error likely occurs...");
    
    let content = fs::read_to_string("examples/LambdaCore-latest.db")?;
    let lines: Vec<&str> = content.lines().collect();
    
    // Find object #0 and examine its verb section
    let mut obj_start = None;
    for (i, line) in lines.iter().enumerate() {
        if line.starts_with("#0") && !line.contains(":") {
            obj_start = Some(i);
            break;
        }
    }
    
    if let Some(start) = obj_start {
        println!("Object #0 starts at line {}", start + 1);
        
        // The object structure should be:
        // Line 11: #0
        // Line 12: The System Object (name)
        // Line 13: (empty handles)
        // Line 14: 24 (flags)
        // Line 15: 2 (owner)  
        // Line 16: -1 (location)
        // Line 17: -1 (contents)
        // Line 18: -1 (next)
        // Line 19: 1 (parent)
        // Line 20: -1 (child)
        // Line 21: 5 (sibling)
        // Line 22: 22 (verb count) <- This is where verb section starts
        
        let verb_section_start = start + 11; // Line 22 (0-indexed)
        println!("\nVerb section starts at line {} with verb count: '{}'", verb_section_start + 1, lines[verb_section_start]);
        
        // Show the verb section structure
        println!("\nVerb section structure:");
        for i in 0..30 {
            if verb_section_start + i < lines.len() {
                let line_num = verb_section_start + i + 1;
                let line = lines[verb_section_start + i];
                println!("{:3}: '{}'", line_num, line);
                
                // Try to identify what each line should be
                if i == 0 {
                    println!("     ^ Should be verb count ({})", line);
                    match line.parse::<i64>() {
                        Ok(count) => println!("     ✅ Verb count: {}", count),
                        Err(e) => println!("     ❌ Invalid verb count: {}", e),
                    }
                } else if i % 4 == 1 {
                    println!("     ^ Should be verb name");
                } else if i % 4 == 2 {
                    println!("     ^ Should be verb owner");
                    if !line.trim().is_empty() {
                        match line.parse::<i64>() {
                            Ok(owner) => println!("     ✅ Owner: {}", owner),
                            Err(e) => println!("     ❌ Invalid owner: {}", e),
                        }
                    }
                } else if i % 4 == 3 {
                    println!("     ^ Should be verb perms");
                    if !line.trim().is_empty() {
                        match line.parse::<i64>() {
                            Ok(perms) => println!("     ✅ Perms: {}", perms),
                            Err(e) => println!("     ❌ Invalid perms: {}", e),
                        }
                    }
                } else if i % 4 == 0 && i > 0 {
                    println!("     ^ Should be verb prep");
                    if !line.trim().is_empty() {
                        match line.parse::<i64>() {
                            Ok(prep) => println!("     ✅ Prep: {}", prep),
                            Err(e) => println!("     ❌ Invalid prep: {}", e),
                        }
                    }
                }
                
                // Stop when we hit property section (starts with a number that's property count)
                if i > 4 && line.chars().all(|c| c.is_ascii_digit()) && lines.get(verb_section_start + i + 1).map_or(false, |next| !next.chars().all(|c| c.is_ascii_digit() || c == '-')) {
                    println!("     ^ Likely start of property section");
                    break;
                }
            }
        }
        
        // Try to parse a small section with verb definitions rule
        let verb_count = lines[verb_section_start].parse::<usize>().unwrap_or(0);
        println!("\nTrying to parse verb definitions section...");
        
        // Calculate expected end of verb section: 1 line for count + 4 lines per verb
        let expected_verb_lines = 1 + (verb_count * 4);
        let verb_section_end = verb_section_start + expected_verb_lines;
        
        if verb_section_end < lines.len() {
            let verb_section: Vec<&str> = lines[verb_section_start..verb_section_end].to_vec();
            let verb_content = verb_section.join("\n") + "\n";
            
            println!("Verb section content ({} lines):", verb_section.len());
            for (i, line) in verb_section.iter().enumerate() {
                println!("  {}: '{}'", i, line);
            }
            
            match LambdaMooDbParser::parse(Rule::verb_definitions, &verb_content) {
                Ok(_) => println!("✅ Verb definitions parse successfully"),
                Err(e) => println!("❌ Verb definitions parsing failed: {}", e),
            }
        }
    }
    
    Ok(())
}