use std::fs;
use std::path::Path;
use std::collections::HashMap;
use echo_core::parser::moo_compat::MooCompatParser;
use echo_core::parser::moo_preprocessor::MooPreprocessor;
use echo_core::parser::Parser;
use echo_core::ast::EchoAst;
use echo_core::storage::object_store::{ObjectStore, ObjectId};

fn main() -> anyhow::Result<()> {
    let moo_dir = "../cowbell/objdump-out/textdump-1743778867.moo";
    
    // Create a temporary database for testing
    let temp_dir = tempfile::tempdir()?;
    let store = ObjectStore::new(temp_dir.path())?;
    
    println!("=== PHASE 1: Loading constants ===\n");
    
    // First, load and preprocess constants
    let mut preprocessor = MooPreprocessor::new();
    let constants_path = Path::new(moo_dir).join("constants.moo");
    if let Ok(content) = fs::read_to_string(&constants_path) {
        preprocessor.load_defines(&content);
        println!("Loaded {} defines", preprocessor.defines().len());
    }
    
    println!("\n=== PHASE 2: Importing objects ===\n");
    
    // List of MOO files in dependency order
    let moo_files = vec![
        "sysobj.moo",       // #0 - System object
        "root.moo",         // #1 - Base object
        "arch_wizard.moo",  // #2
        "room.moo",         // #3
        "player.moo",       // #4
        "builder.moo",      // #5
        "prog.moo",         // #6
        "hacker.moo",       // #7
        "wiz.moo",          // #8
        "string.moo",       // #10
        "password.moo",     // #11
        "first_room.moo",   // #12
        "login.moo",        // #13
        "event.moo",        // #14
        "sub.moo",          // #15
        "block.moo",        // #16
        "look.moo",         // #17
        "list.moo",         // #18
        "thing.moo",        // #19
    ];
    
    let mut total_parsed = 0;
    let mut total_errors = 0;
    
    for file in &moo_files {
        let path = Path::new(moo_dir).join(file);
        print!("Processing {}: ", file);
        
        match fs::read_to_string(&path) {
            Ok(content) => {
                // Preprocess the content
                let processed = preprocessor.process(&content);
                
                // Try to parse the preprocessed content
                let mut parser = MooCompatParser::new()?;
                match parser.parse(&processed) {
                    Ok(ast) => {
                        print!("PARSED - ");
                        
                        // Check what we got
                        match &ast {
                            EchoAst::Program(statements) => {
                                println!("{} statements", statements.len());
                                
                                // Show first few statements
                                for (i, stmt) in statements.iter().take(3).enumerate() {
                                    match stmt {
                                        EchoAst::Identifier(name) => {
                                            println!("  Statement {}: Identifier({})", i, name);
                                        }
                                        EchoAst::Assignment { target, .. } => {
                                            println!("  Statement {}: Assignment to {:?}", i, target);
                                        }
                                        EchoAst::ObjectDef { name, .. } => {
                                            println!("  Statement {}: ObjectDef({})", i, name);
                                        }
                                        _ => {
                                            println!("  Statement {}: {:?}", i, stmt);
                                        }
                                    }
                                }
                                if statements.len() > 3 {
                                    println!("  ... and {} more statements", statements.len() - 3);
                                }
                            }
                            _ => {
                                println!("Got: {:?}", ast);
                            }
                        }
                        
                        total_parsed += 1;
                    }
                    Err(e) => {
                        println!("FAILED - {}", e);
                        total_errors += 1;
                    }
                }
            }
            Err(e) => {
                println!("FAILED - Could not read file: {}", e);
                total_errors += 1;
            }
        }
    }
    
    println!("\n=== SUMMARY ===");
    println!("Total files processed: {}", moo_files.len());
    println!("Successfully parsed: {}", total_parsed);
    println!("Errors: {}", total_errors);
    
    Ok(())
}