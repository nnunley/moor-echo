use anyhow::Result;
use rustyline::DefaultEditor;
use rustyline::error::ReadlineError;
use echo_repl::repl::{Repl, ReplCommand};
use std::io::{self, IsTerminal};

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
    
    let is_interactive = io::stdin().is_terminal();
    let mut in_eval_mode = false;
    let mut eval_buffer = String::new();
    
    while repl.is_running() {
        let prompt = if in_eval_mode { "eval> " } else { "echo> " };
        
        match rl.readline(prompt) {
            Ok(line) => {
                if in_eval_mode {
                    // We're in eval mode, accumulate until we see a single '.'
                    if line.trim() == "." {
                        // End of eval mode, execute the accumulated code
                        rl.add_history_entry(&eval_buffer)?;
                        
                        // Echo input in non-interactive mode
                        if !is_interactive {
                            println!(">> .eval");
                            for eval_line in eval_buffer.lines() {
                                println!(">> {}", eval_line);
                            }
                            println!(">> .");
                        }
                        
                        // Execute the eval buffer as a program
                        match repl.execute_program(&eval_buffer) {
                            Ok((output, duration)) => {
                                println!("=> {} [{:.3}ms]", output, duration.as_secs_f64() * 1000.0);
                            }
                            Err(e) => eprintln!("Error: {}", e),
                        }
                        
                        eval_buffer.clear();
                        in_eval_mode = false;
                    } else {
                        // Add line to eval buffer
                        if !eval_buffer.is_empty() {
                            eval_buffer.push('\n');
                        }
                        eval_buffer.push_str(&line);
                    }
                } else {
                    // Normal single-line mode
                    let trimmed = line.trim();
                    
                    // Handle empty input
                    if trimmed.is_empty() {
                        continue;
                    }
                    
                    // Check if it's the .eval command
                    if trimmed == ".eval" {
                        in_eval_mode = true;
                        println!("Entering eval mode. End with '.' on a line by itself.");
                        continue;
                    }
                    
                    // Normal command processing
                    rl.add_history_entry(&line)?;
                    
                    // Echo input in non-interactive mode
                    if !is_interactive && !trimmed.starts_with('.') {
                        println!(">> {}", trimmed);
                    }
                    
                    match repl.parse_input(&trimmed) {
                        Ok(command) => {
                            match repl.handle_command(command) {
                                Ok(output) => println!("{}", output),
                                Err(e) => eprintln!("Error: {}", e),
                            }
                        }
                        Err(e) => eprintln!("Error: {}", e),
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                if in_eval_mode {
                    // Cancel eval mode
                    println!("^C");
                    eval_buffer.clear();
                    in_eval_mode = false;
                } else {
                    println!("Use .quit to exit");
                }
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
