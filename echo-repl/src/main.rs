use anyhow::Result;
use rustyline::DefaultEditor;
use rustyline::error::ReadlineError;
use echo_repl::repl::{Repl, ReplCommand};

fn main() -> Result<()> {
    env_logger::init();
    
    println!("Echo REPL v0.1.0");
    println!("Type .help for help, .quit to exit");
    println!();
    
    let mut repl = Repl::new();
    let mut rl = DefaultEditor::new()?;
    
    // Create default player
    match repl.handle_command(ReplCommand::CreatePlayer("guest".to_string())) {
        Ok(msg) => println!("{}", msg),
        Err(e) => eprintln!("Error creating default player: {}", e),
    }
    println!();
    
    while repl.is_running() {
        let prompt = "echo> ";
        
        match rl.readline(prompt) {
            Ok(line) => {
                if line.trim().is_empty() {
                    continue;
                }
                
                rl.add_history_entry(&line)?;
                
                match repl.parse_input(&line) {
                    Ok(command) => {
                        match repl.handle_command(command) {
                            Ok(output) => println!("{}", output),
                            Err(e) => eprintln!("Error: {}", e),
                        }
                    }
                    Err(e) => eprintln!("Error: {}", e),
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("Use .quit to exit");
            }
            Err(ReadlineError::Eof) => {
                println!("Goodbye!");
                break;
            }
            Err(err) => {
                eprintln!("Error: {}", err);
                break;
            }
        }
    }
    
    Ok(())
}
