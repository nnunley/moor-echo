use anyhow::Result;
use rustyline::DefaultEditor;
use rustyline::error::ReadlineError;
use echo_repl::repl::{Repl, ReplCommand, MultiLineCollector, LineProcessResult};
use std::io::{self, IsTerminal};
use std::env;
use std::fs;
use std::sync::Arc;

#[cfg(feature = "web-ui")]
use {
    echo_repl::repl::web_notifier::WebNotifier,
    echo_repl::web::WebServer,
    tokio::sync::Mutex as TokioMutex,
};

#[cfg(feature = "web-ui")]
#[tokio::main]
async fn main() -> Result<()> {
    run_main().await
}

#[cfg(not(feature = "web-ui"))]
fn main() -> Result<()> {
    tokio::runtime::Runtime::new()?.block_on(run_main())
}

// Helper function to convert Echo values to JSON
#[cfg(feature = "web-ui")]
fn value_to_json(value: &echo_repl::evaluator::Value) -> serde_json::Value {
    use echo_repl::evaluator::Value;
    
    match value {
        Value::Null => serde_json::Value::Null,
        Value::Boolean(b) => serde_json::Value::Bool(*b),
        Value::Integer(i) => serde_json::Value::Number((*i).into()),
        Value::Float(f) => serde_json::json!(*f),
        Value::String(s) => serde_json::Value::String(s.clone()),
        Value::Object(id) => serde_json::Value::String(id.to_string()),
        Value::List(list) => {
            let json_list: Vec<serde_json::Value> = list.iter()
                .map(|v| value_to_json(v))
                .collect();
            serde_json::Value::Array(json_list)
        },
        Value::Map(map) => {
            let mut json_map = serde_json::Map::new();
            for (k, v) in map {
                json_map.insert(k.clone(), value_to_json(v));
            }
            serde_json::Value::Object(json_map)
        },
        Value::Lambda { .. } => serde_json::Value::String("<lambda>".to_string()),
    }
}

async fn run_main() -> Result<()> {
    env_logger::init();
    
    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    let mut db_path = "./echo-db".to_string();
    let mut input_file = None;
    #[allow(unused_mut)]
    let mut web_ui = false;
    #[allow(unused_mut)]
    let mut web_port = 8080u16;
    
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--db" => {
                if i + 1 < args.len() {
                    db_path = args[i + 1].clone();
                    i += 2;
                } else {
                    eprintln!("Error: --db requires a directory path");
                    std::process::exit(1);
                }
            }
            #[cfg(feature = "web-ui")]
            "--web" => {
                web_ui = true;
                i += 1;
            }
            #[cfg(feature = "web-ui")]
            "--port" => {
                if i + 1 < args.len() {
                    match args[i + 1].parse() {
                        Ok(port) => {
                            web_port = port;
                            i += 2;
                        }
                        Err(_) => {
                            eprintln!("Error: --port requires a valid port number");
                            std::process::exit(1);
                        }
                    }
                } else {
                    eprintln!("Error: --port requires a port number");
                    std::process::exit(1);
                }
            }
            arg => {
                if !arg.starts_with("--") {
                    input_file = Some(arg.to_string());
                }
                i += 1;
            }
        }
    }
    
    // Create database directory if it doesn't exist
    fs::create_dir_all(&db_path)?;
    
    println!("Echo REPL v0.1.0");
    println!("Database: {}", db_path);
    
    #[cfg(feature = "web-ui")]
    if web_ui {
        println!("Web UI: http://localhost:{}", web_port);
        // Create web notifier and REPL
        let notifier = Arc::new(WebNotifier::new(1000));
        let mut repl = echo_repl::repl::Repl::with_storage_path_and_notifier(
            db_path.clone(),
            notifier.clone(),
        )?;
        
        // Subscribe to UI events from the evaluator
        let ui_notifier = notifier.clone();
        repl.evaluator_mut().event_system().subscribe("ui_update".to_string(), move |event| {
            use echo_repl::repl::web_notifier::UiUpdate;
            use echo_repl::evaluator::Value;
            
            if let Some(action) = event.args.get(0) {
                match action {
                    Value::String(action_str) => {
                        let update = match action_str.as_str() {
                            "clear" => UiUpdate {
                                target: "dynamicContent".to_string(),
                                action: "clear".to_string(),
                                data: serde_json::json!({}),
                            },
                            "add_button" => {
                                if let (Some(Value::String(id)), Some(Value::String(label)), Some(Value::String(action))) = 
                                    (event.args.get(1), event.args.get(2), event.args.get(3)) {
                                    UiUpdate {
                                        target: id.clone(),
                                        action: "add_button".to_string(),
                                        data: serde_json::json!({
                                            "label": label,
                                            "action": action,
                                        }),
                                    }
                                } else {
                                    return Ok(());
                                }
                            },
                            "add_text" => {
                                if let (Some(Value::String(id)), Some(Value::String(text))) = 
                                    (event.args.get(1), event.args.get(2)) {
                                    let mut data = serde_json::json!({
                                        "text": text,
                                    });
                                    
                                    // Handle optional style parameter
                                    if let Some(style_val) = event.args.get(3) {
                                        if let Value::Map(style_map) = style_val {
                                            let mut style_obj = serde_json::Map::new();
                                            for (k, v) in style_map {
                                                style_obj.insert(k.clone(), serde_json::Value::String(v.to_string()));
                                            }
                                            data["style"] = serde_json::Value::Object(style_obj);
                                        }
                                    }
                                    
                                    UiUpdate {
                                        target: id.clone(),
                                        action: "add_text".to_string(),
                                        data,
                                    }
                                } else {
                                    return Ok(());
                                }
                            },
                            _ => return Ok(()),
                        };
                        
                        ui_notifier.send_ui_update(update);
                    }
                    _ => {}
                }
            }
            
            Ok(())
        });
        
        // Subscribe to web-targeted events (pattern: "web:*")
        let event_notifier = notifier.clone();
        repl.evaluator_mut().event_system().subscribe("web:*".to_string(), move |event| {
            use echo_repl::repl::web_notifier::EventData;
            use echo_repl::evaluator::Value;
            use std::time::{SystemTime, UNIX_EPOCH};
            
            // Convert event args to JSON values
            let mut json_args = Vec::new();
            for arg in &event.args {
                json_args.push(value_to_json(arg));
            }
            
            // Create event data
            let event_data = EventData {
                name: event.name.clone(),
                args: json_args,
                emitter: event.emitter.to_string(),
                timestamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64,
            };
            
            event_notifier.send_echo_event(event_data);
            Ok(())
        });
        
        // Create or use default player
        match repl.handle_command(ReplCommand::CreatePlayer("guest".to_string())) {
            Ok(msg) => repl.notifier().on_output(&msg),
            Err(e) => {
                // If player already exists, try to use it
                if e.to_string().contains("already exists") {
                    match repl.handle_command(ReplCommand::SwitchPlayer("guest".to_string())) {
                        Ok(msg) => repl.notifier().on_output(&msg),
                        Err(e) => repl.notifier().on_error(&format!("Error switching to guest player: {}", e)),
                    }
                } else {
                    repl.notifier().on_error(&format!("Error creating default player: {}", e));
                }
            }
        }
        repl.notifier().on_output("");
        
        let repl = Arc::new(TokioMutex::new(repl));
        
        // Start web server
        let web_server = WebServer::new(repl.clone(), notifier.clone());
        let app = web_server.routes();
        
        let addr = std::net::SocketAddr::from(([127, 0, 0, 1], web_port));
        let listener = tokio::net::TcpListener::bind(addr).await?;
        
        // If we have an input file, process it first
        if let Some(file) = input_file {
            let mut repl_lock = repl.lock().await;
            process_file(&mut *repl_lock, file)?;
        }
        
        // Run the web server
        println!("Starting web server on http://localhost:{}", web_port);
        println!("Use Ctrl+C to exit");
        
        axum::serve(listener, app).await?;
        
        return Ok(());
    }
    
    println!("Type .help for help, .quit to exit");
    println!();
    
    // Run normal REPL without web UI
    let mut repl = Repl::with_storage_path(db_path)?;
    run_repl_sync(&mut repl, input_file)
}

#[cfg(feature = "web-ui")]
async fn run_repl_loop(
    repl: Arc<TokioMutex<Repl>>,
    input_file: Option<String>,
) -> Result<()> {
    let mut repl = repl.lock().await;
    run_repl_sync(&mut *repl, input_file)
}

fn process_file(repl: &mut Repl, filename: String) -> Result<()> {
    let content = fs::read_to_string(&filename)?;
    println!("Processing file: {}", filename);
    
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        
        println!(">> {}", line);
        
        if trimmed.starts_with('.') {
            match repl.parse_input(trimmed) {
                Ok(command) => {
                    match repl.handle_command(command) {
                        Ok(output) => repl.notifier().on_output(&output),
                        Err(e) => repl.notifier().on_error(&format!("Error: {}", e)),
                    }
                }
                Err(e) => repl.notifier().on_error(&format!("Error: {}", e)),
            }
        } else {
            match repl.execute(line) {
                Ok((output, duration)) => {
                    repl.notifier().on_result(&output, duration, repl.is_quiet());
                }
                Err(e) => repl.notifier().on_error(&format!("Error: {}", e)),
            }
        }
    }
    
    Ok(())
}

fn run_repl_sync(repl: &mut Repl, input_file: Option<String>) -> Result<()> {
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
                    Err(e) => repl.notifier().on_error(&format!("Error switching to guest player: {}", e)),
                }
            } else {
                repl.notifier().on_error(&format!("Error creating default player: {}", e));
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
                                println!(">> {}", eval_line);
                            }
                            println!(">> .");
                        }
                        
                        // Execute the eval buffer as a program
                        match repl.execute_program(&eval_buffer) {
                            Ok((output, duration)) => {
                                repl.notifier().on_result(&output, duration, repl.is_quiet());
                            }
                            Err(e) => repl.notifier().on_error(&format!("Error: {}", e)),
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
                        repl.notifier().on_output("Entering eval mode. End with '.' on a line by itself.");
                        continue;
                    }
                    
                    // Check if it's a REPL command
                    if trimmed.starts_with('.') {
                        rl.add_history_entry(&line)?;
                        match repl.parse_input(&trimmed) {
                            Ok(command) => {
                                match repl.handle_command(command) {
                                    Ok(output) => repl.notifier().on_output(&output),
                                    Err(e) => repl.notifier().on_error(&format!("Error: {}", e)),
                                }
                            }
                            Err(e) => repl.notifier().on_error(&format!("Error: {}", e)),
                        }
                    } else {
                        // Process through multi-line collector
                        match multiline.process_line(&line, repl.parser_mut()) {
                            LineProcessResult::Complete(code) => {
                                rl.add_history_entry(&code)?;
                                
                                // Echo input in non-interactive mode
                                if !is_interactive {
                                    println!(">> {}", code);
                                }
                                
                                // Execute the complete code
                                match repl.execute(&code) {
                                    Ok((output, duration)) => {
                                        repl.notifier().on_result(&output, duration, repl.is_quiet());
                                    }
                                    Err(e) => repl.notifier().on_error(&format!("Error: {}", e)),
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
                eprintln!("Error: {}", err);
                break;
            }
        }
    }
    
    // Show exit statistics
    repl.show_exit_stats();
    
    Ok(())
}