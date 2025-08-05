use std::fs;
use std::path::Path;
use echo_core::parser::moo_compat::MooCompatParser;
use echo_core::parser::Parser;

fn main() {
    let moo_dir = "../cowbell/objdump-out/textdump-1743778867.moo";
    let moo_files = vec![
        "event.moo",
        "login.moo", 
        "sysobj.moo",
        "sub.moo",
        "look.moo",
        "builder.moo",
        "room.moo",
        "wiz.moo",
        "thing.moo",
        "list.moo",
        "first_room.moo",
        "password.moo",
        "prog.moo",
        "constants.moo",
        "block.moo",
        "root.moo",
        "string.moo",
        "hacker.moo",
        "player.moo",
        "arch_wizard.moo",
    ];

    println!("Testing MOO import on {} files...\n", moo_files.len());

    let mut success_count = 0;
    let mut failure_count = 0;
    let mut errors = Vec::new();

    for file in &moo_files {
        let path = Path::new(moo_dir).join(file);
        print!("Testing {}: ", file);
        
        match fs::read_to_string(&path) {
            Ok(content) => {
                let mut parser = match MooCompatParser::new() {
                    Ok(p) => p,
                    Err(e) => {
                        println!("FAILED - Parser creation error: {}", e);
                        failure_count += 1;
                        errors.push((file.to_string(), format!("Parser creation: {}", e)));
                        continue;
                    }
                };
                
                match parser.parse(&content) {
                    Ok(_ast) => {
                        println!("SUCCESS");
                        success_count += 1;
                    }
                    Err(e) => {
                        println!("FAILED - Parse error: {}", e);
                        failure_count += 1;
                        errors.push((file.to_string(), format!("Parse error: {}", e)));
                        
                        // Try to show a snippet of the problematic content
                        if let Some(line_num) = e.to_string().find("line") {
                            let lines: Vec<&str> = content.lines().collect();
                            if lines.len() > 10 {
                                println!("  First 10 lines of file:");
                                for (i, line) in lines.iter().take(10).enumerate() {
                                    println!("  {:3}: {}", i + 1, line);
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => {
                println!("FAILED - File read error: {}", e);
                failure_count += 1;
                errors.push((file.to_string(), format!("File read: {}", e)));
            }
        }
    }

    println!("\n=== SUMMARY ===");
    println!("Total files: {}", moo_files.len());
    println!("Successful: {}", success_count);
    println!("Failed: {}", failure_count);
    
    if !errors.is_empty() {
        println!("\n=== ERRORS ===");
        for (file, error) in &errors {
            println!("{}: {}", file, error);
        }
    }
}