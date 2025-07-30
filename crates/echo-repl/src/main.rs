use std::{
    fs,
    io::{self, IsTerminal},
};

use anyhow::Result;
use clap::{Arg, Command};
use echo_core::{init_tracing, EchoConfig, EchoRuntime};

mod repl;
#[cfg(feature = "web-integration")]
use echo_web::{WebServer, WebServerConfig};
use repl::{LineProcessResult, MultiLineCollector, Repl, ReplCommand};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    init_tracing();

    // Parse command line arguments
    let matches = Command::new("echo-repl")
        .version(echo_core::VERSION)
        .about("Interactive REPL for the Echo programming language")
        .arg(
            Arg::new("database")
                .long("db")
                .value_name("PATH")
                .help("Database directory path")
                .default_value("./echo-db"),
        )
        .arg(
            Arg::new("file")
                .value_name("FILE")
                .help("Execute Echo script file on startup")
                .index(1),
        )
        .arg(
            Arg::new("debug")
                .long("debug")
                .help("Enable debug mode")
                .action(clap::ArgAction::SetTrue),
        );

    #[cfg(feature = "web-integration")]
    let matches = matches
        .arg(
            Arg::new("web")
                .long("web")
                .help("Enable web interface")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("port")
                .long("port")
                .value_name("PORT")
                .help("Web server port")
                .default_value("8080"),
        );

    let matches = matches.get_matches();

    // Extract command line options
    let db_path = matches.get_one::<String>("database").unwrap().clone();
    let input_file = matches.get_one::<String>("file").cloned();
    let debug = matches.get_flag("debug");

    #[cfg(feature = "web-integration")]
    let web_enabled = matches.get_flag("web");
    #[cfg(feature = "web-integration")]
    let web_port = matches.get_one::<String>("port").unwrap().parse::<u16>()?;

    // Create database directory if it doesn't exist
    fs::create_dir_all(&db_path)?;

    println!("Echo REPL v{}", echo_core::VERSION);
    println!("Database: {db_path}");

    if debug {
        println!("Debug mode: enabled");
    }

    // Configure Echo runtime
    let config = EchoConfig {
        storage_path: db_path.into(),
        debug,
        ..Default::default()
    };

    // Create Echo runtime
    let runtime = EchoRuntime::new(config)?;

    // Create REPL with the runtime
    let mut repl = Repl::new(runtime)?;

    #[cfg(feature = "web-integration")]
    if web_enabled {
        println!("Starting web interface on port {}", web_port);

        // Create web server config
        let web_config = WebServerConfig {
            host: "127.0.0.1".to_string(),
            port: web_port,
            static_dir: "./static".into(),
            enable_cors: true,
        };

        // For web mode, we'd need to restructure this to run both
        // the REPL and web server concurrently
        println!("Web integration not fully implemented in this modular version yet");
        println!("Please run without --web flag for now");
        return Ok(());
    }

    println!("Type .help for help, .quit to exit");
    println!();

    // Run REPL
    run_repl(&mut repl, input_file).await
}

async fn run_repl(repl: &mut Repl, input_file: Option<String>) -> Result<()> {
    use rustyline::{error::ReadlineError, DefaultEditor};

    let mut rl = DefaultEditor::new()?;
    let mut multiline = MultiLineCollector::new();

    // Create or use default player
    match repl.handle_command(ReplCommand::CreatePlayer("guest".to_string())) {
        Ok(msg) => repl.notifier().on_output(&msg),
        Err(e) => {
            // If player already exists, try to use it
            if e.to_string().contains("already exists") {
                match repl.handle_command(ReplCommand::SwitchPlayer("guest".to_string())) {
                    Ok(msg) => repl.notifier().on_output(&msg),
                    Err(e) => repl
                        .notifier()
                        .on_error(&format!("Error switching to guest player: {e}")),
                }
            } else {
                repl.notifier()
                    .on_error(&format!("Error creating default player: {e}"));
            }
        }
    }
    repl.notifier().on_output("");

    // Check if we have a file argument
    let file_lines: Option<Vec<String>> = if let Some(filename) = input_file {
        let content = fs::read_to_string(filename)?;
        Some(content.lines().map(|s| s.to_string()).collect())
    } else {
        None
    };

    let is_interactive = file_lines.is_none() && io::stdin().is_terminal();
    let mut file_line_iter = file_lines.as_ref().map(|lines| lines.iter());
    let mut in_eval_mode = false;
    let mut eval_buffer = String::new();
    let _command_history: Vec<String> = Vec::new();
    let _history_index = 0;

    while repl.is_running() {
        let prompt = if in_eval_mode {
            ">> "
        } else {
            multiline.get_prompt()
        };

        // Get the next line from either file or interactive input
        let line_result = if let Some(ref mut iter) = file_line_iter {
            if let Some(line) = iter.next() {
                Ok(line.clone())
            } else {
                // End of file
                break;
            }
        } else {
            rl.readline(prompt)
        };

        match line_result {
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
                                println!(">> {eval_line}");
                            }
                            println!(">> .");
                        }

                        // Execute the eval buffer as a program
                        match repl.execute_program(&eval_buffer) {
                            Ok((output, duration)) => {
                                repl.notifier()
                                    .on_result(&output, duration, repl.is_quiet());
                            }
                            Err(e) => repl.notifier().on_error(&format!("Error: {e}")),
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
                        repl.notifier()
                            .on_output("Entering eval mode. End with '.' on a line by itself.");
                        continue;
                    }

                    // Check if it's a REPL command
                    if trimmed.starts_with('.') {
                        rl.add_history_entry(&line)?;
                        match repl.parse_input(trimmed) {
                            Ok(command) => match repl.handle_command(command) {
                                Ok(output) => repl.notifier().on_output(&output),
                                Err(e) => repl.notifier().on_error(&format!("Error: {e}")),
                            },
                            Err(e) => repl.notifier().on_error(&format!("Error: {e}")),
                        }
                    } else {
                        // Process through multi-line collector
                        match multiline.process_line(&line, repl.parser_mut()) {
                            LineProcessResult::Complete(code) => {
                                rl.add_history_entry(&code)?;

                                // Echo input in non-interactive mode
                                if !is_interactive {
                                    println!(">> {code}");
                                }

                                // Execute the complete code
                                match repl.execute(&code) {
                                    Ok((output, duration)) => {
                                        repl.notifier().on_result(
                                            &output,
                                            duration,
                                            repl.is_quiet(),
                                        );
                                    }
                                    Err(e) => repl.notifier().on_error(&format!("Error: {e}")),
                                }
                            }
                            LineProcessResult::NeedMore => {
                                // Continue collecting lines
                            }
                        }
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                if in_eval_mode {
                    // Cancel eval mode
                    println!("^C");
                    eval_buffer.clear();
                    in_eval_mode = false;
                } else if multiline.is_collecting() {
                    // Cancel multi-line collection
                    println!("^C");
                    multiline.reset();
                } else {
                    println!("Use .quit to exit");
                }
            }
            Err(ReadlineError::Eof) => {
                println!("Goodbye!");
                break;
            }
            Err(err) => {
                eprintln!("Error: {err}");
                break;
            }
        }
    }

    // Show exit statistics
    repl.show_exit_stats();

    Ok(())
}
