use thiserror::Error;
use std::sync::Arc;
use std::path::PathBuf;
use anyhow::Result;
use std::time::Instant;
use std::time::Duration;

use crate::parser::{create_parser, Parser};
use crate::ast::EchoAst;
use crate::evaluator::Evaluator;
use crate::storage::Storage;

mod multiline_simple;
pub use multiline_simple::{MultiLineCollector, LineProcessResult};

#[cfg(feature = "web-ui")]
pub mod web_notifier;

/// Trait for handling REPL output notifications
pub trait ReplNotifier: Send + Sync {
    /// Called when a command or expression has been evaluated
    fn on_result(&self, value: &str, duration: Duration, quiet_mode: bool);
    
    /// Called for general output messages
    fn on_output(&self, message: &str);
    
    /// Called for error messages
    fn on_error(&self, error: &str);
}

/// Default notifier that prints to stdout/stderr
pub struct DefaultNotifier;

impl ReplNotifier for DefaultNotifier {
    fn on_result(&self, value: &str, duration: Duration, quiet_mode: bool) {
        if !quiet_mode {
            println!("=> {} [{:.3}ms]", value, duration.as_secs_f64() * 1000.0);
        }
    }
    
    fn on_output(&self, message: &str) {
        println!("{}", message);
    }
    
    fn on_error(&self, error: &str) {
        eprintln!("{}", error);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ReplCommand {
    Help,
    Quit,
    Execute(String),
    Eval,  // Start multi-line evaluation mode
    // Player management commands
    CreatePlayer(String),
    SwitchPlayer(String),
    ListPlayers,
    CurrentPlayer,
    // Environment and scope commands
    ShowEnvironment,
    ShowScope,
    // Dump environment as JSON to stderr
    DumpEnvironment,
    // Toggle quiet mode (suppress evaluation output)
    ToggleQuiet,
    // Load and execute a file
    Load(String),
    // Reset the current environment
    Reset,
    // Show session statistics
    Stats,
}

#[derive(Error, Debug, PartialEq)]
pub enum ReplError {
    #[error("Parse error: {0}")]
    ParseError(String),
    
    #[error("Execution error: {0}")]
    ExecutionError(String),
    
    #[error("Unknown command: {0}")]
    UnknownCommand(String),
    
    #[error("Storage error: {0}")]
    StorageError(String),
    
    #[error("No player selected")]
    NoPlayer,
}

pub struct Repl {
    running: bool,
    parser: Box<dyn Parser>,
    evaluator: Evaluator,
    storage: Arc<Storage>,
    quiet_mode: bool,
    notifier: Arc<dyn ReplNotifier>,
    // Session statistics
    session_start: Instant,
    eval_count: usize,
    total_eval_time: Duration,
    // Multi-line eval mode state
    eval_mode: bool,
    eval_buffer: String,
}

impl Repl {
    pub fn new() -> Self {
        Self::with_storage_path("./echo-db").expect("Failed to create REPL")
    }
    
    pub fn with_storage_path(path: impl Into<PathBuf>) -> Result<Self> {
        Self::with_storage_path_and_notifier(path, Arc::new(DefaultNotifier))
    }
    
    pub fn with_storage_path_and_notifier(
        path: impl Into<PathBuf>, 
        notifier: Arc<dyn ReplNotifier>
    ) -> Result<Self> {
        let storage = Arc::new(Storage::new(path.into())?);
        let parser = create_parser("echo")?;
        let evaluator = Evaluator::new(storage.clone());
        
        Ok(Self {
            running: true,
            parser,
            evaluator,
            storage,
            quiet_mode: false,
            notifier,
            session_start: Instant::now(),
            eval_count: 0,
            total_eval_time: Duration::ZERO,
            eval_mode: false,
            eval_buffer: String::new(),
        })
    }
    
    pub fn is_running(&self) -> bool {
        self.running
    }
    
    pub fn is_quiet(&self) -> bool {
        self.quiet_mode
    }
    
    pub fn notifier(&self) -> &Arc<dyn ReplNotifier> {
        &self.notifier
    }
    
    pub fn show_exit_stats(&self) {
        if self.eval_count > 0 {
            let session_duration = self.session_start.elapsed();
            let avg_eval_time = self.total_eval_time / self.eval_count as u32;
            
            self.notifier.on_output(&format!(
                "\nTotal Time: {:.3}ms, Cumulative Evaluation: {:.3}ms, Per Evaluation: {:.3}µs",
                session_duration.as_secs_f64() * 1000.0,
                self.total_eval_time.as_secs_f64() * 1000.0,
                avg_eval_time.as_micros() as f64
            ));
        }
    }
    
    pub fn parse_input(&self, input: &str) -> Result<ReplCommand, ReplError> {
        let trimmed = input.trim();
        
        if trimmed.starts_with('.') {
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            match parts.as_slice() {
                [".help"] => Ok(ReplCommand::Help),
                [".quit"] => Ok(ReplCommand::Quit),
                [".eval"] => Ok(ReplCommand::Eval),
                [".env"] => Ok(ReplCommand::ShowEnvironment),
                [".scope"] => Ok(ReplCommand::ShowScope),
                [".dump"] => Ok(ReplCommand::DumpEnvironment),
                [".quiet"] => Ok(ReplCommand::ToggleQuiet),
                [".reset"] => Ok(ReplCommand::Reset),
                [".stats"] => Ok(ReplCommand::Stats),
                [".player", "create", name] => Ok(ReplCommand::CreatePlayer(name.to_string())),
                [".player", "switch", name] => Ok(ReplCommand::SwitchPlayer(name.to_string())),
                [".player", "list"] => Ok(ReplCommand::ListPlayers),
                [".player"] => Ok(ReplCommand::CurrentPlayer),
                _ => {
                    // Check for .load command with filename
                    if trimmed.starts_with(".load ") {
                        let filename = trimmed[6..].trim();
                        if filename.is_empty() {
                            Err(ReplError::ParseError("Usage: .load <filename>".to_string()))
                        } else {
                            Ok(ReplCommand::Load(filename.to_string()))
                        }
                    } else {
                        Err(ReplError::UnknownCommand(trimmed.to_string()))
                    }
                }
            }
        } else {
            Ok(ReplCommand::Execute(trimmed.to_string()))
        }
    }
    
    fn execute_internal<F>(&mut self, code: &str, parse_fn: F) -> Result<(String, Duration), ReplError>
    where
        F: FnOnce(&mut dyn Parser, &str) -> Result<EchoAst, anyhow::Error>,
    {
        // Start timing
        let start = Instant::now();
        
        // Parse the code using the provided parse function
        let ast = parse_fn(&mut *self.parser, code)
            .map_err(|e| ReplError::ParseError(e.to_string()))?;
        
        // Ensure we have a player (create default if needed)
        if self.evaluator.current_player().is_none() {
            // Create a default player
            let player_id = self.evaluator.create_player("default")
                .map_err(|e| ReplError::StorageError(e.to_string()))?;
            self.evaluator.switch_player(player_id)
                .map_err(|e| ReplError::StorageError(e.to_string()))?;
        }
        
        // Evaluate the AST
        let result = self.evaluator.eval(&ast)
            .map_err(|e| ReplError::ExecutionError(e.to_string()))?;
        
        let elapsed = start.elapsed();
        
        // Update statistics
        self.eval_count += 1;
        self.total_eval_time += elapsed;
        
        // Handle special cases for object creation
        let output = if matches!(ast, EchoAst::ObjectDef { .. }) {
            "object created".to_string()
        } else {
            result.to_string()
        };
        
        Ok((output, elapsed))
    }
    
    pub fn execute(&mut self, code: &str) -> Result<(String, Duration), ReplError> {
        self.execute_internal(code, |parser, code| parser.parse(code))
    }
    
    pub fn execute_program(&mut self, code: &str) -> Result<(String, Duration), ReplError> {
        self.execute_internal(code, |parser, code| parser.parse_program(code))
    }
    
    pub fn parser_mut(&mut self) -> &mut dyn Parser {
        &mut *self.parser
    }
    
    pub fn current_player_name(&self) -> Option<String> {
        self.evaluator.current_player_name()
    }
    
    pub fn get_environment_snapshot(&self) -> Vec<(String, String, String)> {
        if let Some(player_id) = self.evaluator.current_player() {
            if let Some(env) = self.evaluator.get_environment(player_id) {
                let mut vars = Vec::new();
                for (name, val) in &env.variables {
                    vars.push((
                        name.clone(),
                        format!("{}", val),
                        val.type_name().to_string(),
                    ));
                }
                return vars;
            }
        }
        Vec::new()
    }
    
    pub fn get_objects_snapshot(&self) -> Vec<(String, String, Vec<String>)> {
        let objects = Vec::new();
        // This is a simplified version - you'd need to implement proper object listing
        // For now, return empty vec
        objects
    }
    
    pub fn is_eval_mode(&self) -> bool {
        self.eval_mode
    }
    
    pub fn get_prompt(&self) -> &'static str {
        if self.eval_mode {
            ">> "
        } else {
            "echo> "
        }
    }
    
    pub fn process_input(&mut self, line: &str) -> Result<Option<(String, Duration)>, ReplError> {
        if self.eval_mode {
            // We're in eval mode, accumulate until we see a single '.'
            if line.trim() == "." {
                // End of eval mode, execute the accumulated code
                let code = std::mem::take(&mut self.eval_buffer);
                self.eval_mode = false;
                
                // Execute the eval buffer as a program
                self.execute_program(&code).map(Some)
            } else {
                // Add line to eval buffer
                if !self.eval_buffer.is_empty() {
                    self.eval_buffer.push('\n');
                }
                self.eval_buffer.push_str(line);
                Ok(None)
            }
        } else {
            // Normal mode - check for commands first
            let trimmed = line.trim();
            
            if trimmed.is_empty() {
                return Ok(None);
            }
            
            if trimmed.starts_with('.') {
                let command = self.parse_input(trimmed)?;
                
                if matches!(command, ReplCommand::Eval) {
                    self.eval_mode = true;
                    self.eval_buffer.clear();
                    self.notifier.on_output("Entering eval mode. End with '.' on a line by itself.");
                    Ok(None)
                } else {
                    let output = self.handle_command(command)?;
                    self.notifier.on_output(&output);
                    Ok(None)
                }
            } else {
                // Execute as a single line
                self.execute(trimmed).map(Some)
            }
        }
    }
    
    pub fn cancel_eval_mode(&mut self) {
        if self.eval_mode {
            self.eval_mode = false;
            self.eval_buffer.clear();
        }
    }
    
    pub fn handle_command(&mut self, command: ReplCommand) -> Result<String, ReplError> {
        match command {
            ReplCommand::Help => Ok(self.show_help()),
            ReplCommand::Quit => {
                self.running = false;
                Ok("Goodbye!".to_string())
            }
            ReplCommand::Eval => {
                // This should be handled by process_input instead
                Err(ReplError::ExecutionError("Eval command should be handled by process_input".to_string()))
            }
            ReplCommand::Execute(code) => {
                let (output, duration) = self.execute(&code)?;
                Ok(format!("=> {} [{:.3}ms]", output, duration.as_secs_f64() * 1000.0))
            }
            ReplCommand::CreatePlayer(name) => {
                let player_id = self.evaluator.create_player(&name)
                    .map_err(|e| ReplError::StorageError(e.to_string()))?;
                self.evaluator.switch_player(player_id)
                    .map_err(|e| ReplError::StorageError(e.to_string()))?;
                Ok(format!("Created and switched to player '{}' ({})", name, player_id))
            }
            ReplCommand::SwitchPlayer(name) => {
                // Look up player by username in the registry
                match self.evaluator.find_player_by_username(&name) {
                    Ok(Some(player_id)) => {
                        self.evaluator.switch_player(player_id)
                            .map_err(|e| ReplError::StorageError(e.to_string()))?;
                        Ok(format!("Switched to player '{}' ({})", name, player_id))
                    }
                    Ok(None) => Err(ReplError::ExecutionError(format!("Player '{}' not found", name))),
                    Err(e) => Err(ReplError::StorageError(e.to_string())),
                }
            }
            ReplCommand::ListPlayers => {
                // Get the player registry from #0
                let system_obj = self.storage.objects.get(crate::storage::ObjectId::system())
                    .map_err(|e| ReplError::StorageError(e.to_string()))?;
                
                let mut player_list = Vec::new();
                
                // Check if player_registry exists
                if let Some(crate::storage::PropertyValue::Map(registry)) = system_obj.properties.get("player_registry") {
                    // Sort players by username
                    let mut entries: Vec<_> = registry.iter().collect();
                    entries.sort_by_key(|(username, _)| username.as_str());
                    
                    for (username, player_ref) in entries {
                        if let crate::storage::PropertyValue::Object(player_id) = player_ref {
                            // Get player's display name if different from username
                            if let Ok(player_obj) = self.storage.objects.get(*player_id) {
                                let display_name = match player_obj.properties.get("display_name") {
                                    Some(crate::storage::PropertyValue::String(name)) => name,
                                    _ => username,
                                };
                                if display_name != username {
                                    player_list.push(format!("  {} ({}, {}) ", username, display_name, player_id));
                                } else {
                                    player_list.push(format!("  {} ({})", username, player_id));
                                }
                            } else {
                                player_list.push(format!("  {} ({}) [invalid]", username, player_id));
                            }
                        }
                    }
                }
                
                if player_list.is_empty() {
                    Ok("No players found".to_string())
                } else {
                    Ok(format!("Players:\n{}", player_list.join("\n")))
                }
            }
            ReplCommand::CurrentPlayer => {
                match self.evaluator.current_player() {
                    Some(id) => {
                        if let Ok(obj) = self.storage.objects.get(id) {
                            // Get the username and display name from properties
                            let username = match obj.properties.get("username") {
                                Some(crate::storage::PropertyValue::String(name)) => name.clone(),
                                _ => "unknown".to_string(),
                            };
                            let display_name = match obj.properties.get("display_name") {
                                Some(crate::storage::PropertyValue::String(name)) => name.clone(),
                                _ => username.clone(),
                            };
                            
                            if display_name != username {
                                Ok(format!("Current player: {} ({}, {})", username, display_name, id))
                            } else {
                                Ok(format!("Current player: {} ({})", username, id))
                            }
                        } else {
                            Ok(format!("Current player: {}", id))
                        }
                    }
                    None => Ok("No player selected".to_string()),
                }
            }
            ReplCommand::ShowEnvironment => {
                self.show_environment()
            }
            ReplCommand::ShowScope => {
                self.show_scope()
            }
            ReplCommand::DumpEnvironment => {
                self.dump_environment()
            }
            ReplCommand::ToggleQuiet => {
                self.quiet_mode = !self.quiet_mode;
                Ok(format!("Quiet mode: {}", if self.quiet_mode { "ON" } else { "OFF" }))
            }
            ReplCommand::Load(filename) => {
                self.load_file(&filename)
            }
            ReplCommand::Reset => {
                self.reset_environment()
            }
            ReplCommand::Stats => {
                self.show_stats()
            }
        }
    }
    
    fn show_help(&self) -> String {
        r#"Echo REPL Commands:
  .help                  Show this help message
  .quit                  Exit the REPL
  .eval                  Enter multi-line evaluation mode (end with '.')
  .player create <name>  Create a new player
  .player switch <name>  Switch to a different player
  .player list          List all players
  .player               Show current player
  .env                  Show current environment variables
  .scope                Show all variables and objects in scope
  .dump                 Dump environment as JSON to stderr
  .quiet                Toggle quiet mode (suppress evaluation output)
  .load <filename>      Load and execute an Echo file
  .reset                Reset the current environment (clear all variables)
  .stats                Show session statistics
  
Echo Language:
  2 + 2                 Arithmetic expressions
  3.14 + 2.86          Float arithmetic
  #0                    System object reference
  $system               System property (returns #0)
  
Multi-line example (.eval):
  object hello
    property greeting = "Hello";
    property name = "World"
  endobject
  hello
  hello.greeting
  ."#.to_string()
    }
    
    fn show_environment(&self) -> Result<String, ReplError> {
        match self.evaluator.get_current_environment() {
            Some(env) => {
                let mut output = String::new();
                output.push_str("=== Current Environment ===\n");
                output.push_str(&format!("Player: {}\n", env.player_id));
                output.push_str("\nVariables:\n");
                
                if env.variables.is_empty() {
                    output.push_str("  (none)\n");
                } else {
                    let mut vars: Vec<_> = env.variables.iter().collect();
                    vars.sort_by_key(|(k, _)| k.as_str());
                    
                    for (name, value) in vars {
                        let is_const = env.const_bindings.contains(name);
                        let const_marker = if is_const { " (const)" } else { "" };
                        let value_str = value.display_truncated(50);
                        output.push_str(&format!("  {}{} = {}\n", name, const_marker, value_str));
                    }
                }
                
                Ok(output)
            }
            None => Err(ReplError::NoPlayer),
        }
    }
    
    fn show_scope(&self) -> Result<String, ReplError> {
        let mut output = String::new();
        output.push_str("=== Current Scope ===\n");
        
        // Show player environment variables
        if let Some(env) = self.evaluator.get_current_environment() {
            output.push_str(&format!("\nPlayer Variables (player_{}):\n", env.player_id));
            
            if env.variables.is_empty() {
                output.push_str("  (none)\n");
            } else {
                let mut vars: Vec<_> = env.variables.iter().collect();
                vars.sort_by_key(|(k, _)| k.as_str());
                
                for (name, value) in vars {
                    let is_const = env.const_bindings.contains(name);
                    let const_marker = if is_const { " (const)" } else { "" };
                    let value_str = value.display_truncated(50);
                    output.push_str(&format!("  {}{} = {}\n", name, const_marker, value_str));
                }
            }
        }
        
        // Show objects bound to #0
        output.push_str("\nObjects (bound to #0):\n");
        
        match self.storage.objects.get(crate::storage::ObjectId::system()) {
            Ok(system_obj) => {
                let mut objects: Vec<_> = system_obj.properties.iter()
                    .filter_map(|(name, prop)| {
                        if let crate::storage::PropertyValue::Object(id) = prop {
                            Some((name, id))
                        } else {
                            None
                        }
                    })
                    .collect();
                    
                if objects.is_empty() {
                    output.push_str("  (none)\n");
                } else {
                    objects.sort_by_key(|(k, _)| k.as_str());
                    
                    for (name, id) in objects {
                        output.push_str(&format!("  {} = {}\n", name, id));
                    }
                }
            }
            Err(e) => {
                output.push_str(&format!("  (error reading system object: {})\n", e));
            }
        }
        
        // Show system properties
        output.push_str("\nSystem Properties:\n");
        output.push_str("  $system = #0\n");
        output.push_str("  $root = #1\n");
        
        Ok(output)
    }
    
    fn dump_environment(&self) -> Result<String, ReplError> {
        use serde_json::json;
        
        // Collect environment data
        let mut env_data = json!({
            "player": null,
            "variables": {},
            "objects": {},
            "system_properties": {
                "$system": "#0",
                "$root": "#1"
            }
        });
        
        // Get player environment
        if let Some(env) = self.evaluator.get_current_environment() {
            env_data["player"] = json!(env.player_id.to_string());
            
            // Collect variables
            let mut vars = serde_json::Map::new();
            for (name, value) in &env.variables {
                let is_const = env.const_bindings.contains(name);
                vars.insert(name.clone(), json!({
                    "value": value.to_string(),
                    "type": value.type_name(),
                    "const": is_const
                }));
            }
            env_data["variables"] = json!(vars);
        }
        
        // Collect objects bound to #0
        if let Ok(system_obj) = self.storage.objects.get(crate::storage::ObjectId::system()) {
            let mut objects = serde_json::Map::new();
            for (name, prop) in &system_obj.properties {
                if let crate::storage::PropertyValue::Object(id) = prop {
                    objects.insert(name.clone(), json!(id.to_string()));
                }
            }
            env_data["objects"] = json!(objects);
        }
        
        // Write JSON to stderr
        let json_str = serde_json::to_string_pretty(&env_data)
            .map_err(|e| ReplError::ExecutionError(format!("Failed to serialize environment: {}", e)))?;
        
        eprintln!("{}", json_str);
        
        Ok("Environment dumped to stderr".to_string())
    }
    
    fn load_file(&mut self, filename: &str) -> Result<String, ReplError> {
        use std::fs;
        
        // Read the file
        let content = fs::read_to_string(filename)
            .map_err(|e| ReplError::ExecutionError(format!("Failed to read file '{}': {}", filename, e)))?;
        
        // Track results
        let mut _total_lines = 0;
        let mut executed_lines = 0;
        let mut errors = Vec::new();
        
        // Process each line
        for (line_num, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            
            // Skip empty lines and comments (even though parser doesn't support them yet)
            if trimmed.is_empty() || trimmed.starts_with("//") {
                continue;
            }
            
            _total_lines += 1;
            
            // Check if it's a command
            if trimmed.starts_with('.') {
                // Parse and handle the command
                match self.parse_input(trimmed) {
                    Ok(command) => {
                        // Don't allow nested .load commands
                        if matches!(command, ReplCommand::Load(_)) {
                            errors.push(format!("Line {}: Nested .load commands are not allowed", line_num + 1));
                            continue;
                        }
                        
                        match self.handle_command(command) {
                            Ok(output) => {
                                // Use the notifier to output command results
                                self.notifier.on_output(&output);
                                executed_lines += 1;
                            }
                            Err(e) => {
                                errors.push(format!("Line {}: {}", line_num + 1, e));
                            }
                        }
                    }
                    Err(e) => {
                        errors.push(format!("Line {}: {}", line_num + 1, e));
                    }
                }
            } else {
                // Execute as Echo code
                match self.execute(trimmed) {
                    Ok((output, duration)) => {
                        self.notifier.on_result(&output, duration, self.quiet_mode);
                        executed_lines += 1;
                    }
                    Err(e) => {
                        errors.push(format!("Line {}: {}", line_num + 1, e));
                    }
                }
            }
        }
        
        // Build result message
        let mut result = format!("Loaded '{}': {} lines executed", filename, executed_lines);
        
        if !errors.is_empty() {
            result.push_str(&format!(", {} errors", errors.len()));
            if errors.len() <= 5 {
                result.push_str(":\n");
                for error in &errors {
                    result.push_str(&format!("  {}\n", error));
                }
            } else {
                result.push_str(&format!(" (showing first 5):\n"));
                for error in errors.iter().take(5) {
                    result.push_str(&format!("  {}\n", error));
                }
            }
        }
        
        Ok(result)
    }
    
    fn reset_environment(&mut self) -> Result<String, ReplError> {
        // Get current player
        if let Some(player_id) = self.evaluator.current_player() {
            // Clear the environment for this player
            self.evaluator.reset_player_environment(player_id)
                .map_err(|e| ReplError::ExecutionError(format!("Failed to reset environment: {}", e)))?;
            Ok("Environment reset".to_string())
        } else {
            Err(ReplError::NoPlayer)
        }
    }
    
    fn show_stats(&self) -> Result<String, ReplError> {
        let session_duration = self.session_start.elapsed();
        let avg_eval_time = if self.eval_count > 0 {
            self.total_eval_time / self.eval_count as u32
        } else {
            Duration::ZERO
        };
        
        let mut output = String::new();
        output.push_str("=== Session Statistics ===\n\n");
        output.push_str(&format!("Session Duration: {:.3}ms\n", session_duration.as_secs_f64() * 1000.0));
        output.push_str(&format!("Total Evaluation Time: {:.3}ms\n", self.total_eval_time.as_secs_f64() * 1000.0));
        output.push_str(&format!("Number of Evaluations: {}\n", self.eval_count));
        
        if self.eval_count > 0 {
            output.push_str(&format!("Average per Evaluation: {:.3}µs\n", avg_eval_time.as_micros() as f64));
            
            // Calculate overhead (session time - evaluation time)
            let overhead = session_duration.saturating_sub(self.total_eval_time);
            output.push_str(&format!("REPL Overhead: {:.3}ms ({:.1}%)\n", 
                overhead.as_secs_f64() * 1000.0,
                (overhead.as_secs_f64() / session_duration.as_secs_f64()) * 100.0
            ));
        }
        
        // Add memory statistics
        output.push_str("\n=== Memory Usage ===\n\n");
        
        // Get current process memory usage using platform-specific methods
        #[cfg(target_os = "linux")]
        {
            if let Ok(status) = std::fs::read_to_string("/proc/self/status") {
                for line in status.lines() {
                    if line.starts_with("VmRSS:") {
                        if let Some(rss) = line.split_whitespace().nth(1) {
                            if let Ok(kb) = rss.parse::<f64>() {
                                output.push_str(&format!("Process Memory (RSS): {:.2} MB\n", kb / 1024.0));
                            }
                        }
                    } else if line.starts_with("VmSize:") {
                        if let Some(size) = line.split_whitespace().nth(1) {
                            if let Ok(kb) = size.parse::<f64>() {
                                output.push_str(&format!("Virtual Memory: {:.2} MB\n", kb / 1024.0));
                            }
                        }
                    }
                }
            }
        }
        
        #[cfg(target_os = "macos")]
        {
            use std::mem;
            
            #[repr(C)]
            struct RUsage {
                ru_utime: libc::timeval,
                ru_stime: libc::timeval,
                ru_maxrss: i64,
                // ... other fields we don't need
            }
            
            let mut usage: RUsage = unsafe { mem::zeroed() };
            let result = unsafe { libc::getrusage(libc::RUSAGE_SELF, &mut usage as *mut _ as *mut libc::rusage) };
            
            if result == 0 {
                // On macOS, ru_maxrss is in bytes
                let mb = usage.ru_maxrss as f64 / (1024.0 * 1024.0);
                output.push_str(&format!("Process Memory (RSS): {:.2} MB\n", mb));
            }
        }
        
        #[cfg(not(any(target_os = "linux", target_os = "macos")))]
        {
            output.push_str("Memory statistics not available on this platform\n");
        }
        
        // Get object store statistics
        if let Some(player_id) = self.evaluator.current_player() {
            let env = self.evaluator.get_environment(player_id);
            if let Some(env) = env {
                output.push_str(&format!("Environment Variables: {}\n", env.variables.len()));
                output.push_str(&format!("Const Bindings: {}\n", env.const_bindings.len()));
            }
        }
        
        // Get approximate storage stats
        if let Ok(db_size) = self.storage.estimated_size() {
            output.push_str(&format!("Database Size: {:.2} MB\n", db_size as f64 / (1024.0 * 1024.0)));
        }
        
        Ok(output)
    }
}