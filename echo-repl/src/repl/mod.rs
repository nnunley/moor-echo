use thiserror::Error;
use std::sync::Arc;
use std::path::PathBuf;
use anyhow::Result;
use std::time::Instant;

use crate::parser::{create_parser, Parser};
use crate::ast::EchoAst;
use crate::evaluator::{Evaluator, Value};
use crate::storage::Storage;

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
}

impl Repl {
    pub fn new() -> Self {
        Self::with_storage_path("./echo-db").expect("Failed to create REPL")
    }
    
    pub fn with_storage_path(path: impl Into<PathBuf>) -> Result<Self> {
        let storage = Arc::new(Storage::new(path.into())?);
        let parser = create_parser("echo")?;
        let evaluator = Evaluator::new(storage.clone());
        
        Ok(Self {
            running: true,
            parser,
            evaluator,
            storage,
        })
    }
    
    pub fn is_running(&self) -> bool {
        self.running
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
                [".player", "create", name] => Ok(ReplCommand::CreatePlayer(name.to_string())),
                [".player", "switch", name] => Ok(ReplCommand::SwitchPlayer(name.to_string())),
                [".player", "list"] => Ok(ReplCommand::ListPlayers),
                [".player"] => Ok(ReplCommand::CurrentPlayer),
                _ => Err(ReplError::UnknownCommand(trimmed.to_string())),
            }
        } else {
            Ok(ReplCommand::Execute(trimmed.to_string()))
        }
    }
    
    pub fn execute(&mut self, code: &str) -> Result<(String, std::time::Duration), ReplError> {
        // Start timing
        let start = Instant::now();
        
        // Parse the code
        let ast = self.parser.parse(code)
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
        
        // Handle special cases for object creation
        let output = if matches!(ast, EchoAst::ObjectDef { .. }) {
            "object created".to_string()
        } else {
            result.to_string()
        };
        
        Ok((output, elapsed))
    }
    
    pub fn execute_program(&mut self, code: &str) -> Result<(String, std::time::Duration), ReplError> {
        // Start timing
        let start = Instant::now();
        
        // Parse the code as a program
        let ast = self.parser.parse_program(code)
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
        
        // Handle special cases for object creation
        let output = if matches!(ast, EchoAst::ObjectDef { .. }) {
            "object created".to_string()
        } else {
            result.to_string()
        };
        
        Ok((output, elapsed))
    }
    
    pub fn handle_command(&mut self, command: ReplCommand) -> Result<String, ReplError> {
        match command {
            ReplCommand::Help => Ok(self.show_help()),
            ReplCommand::Quit => {
                self.running = false;
                Ok("Goodbye!".to_string())
            }
            ReplCommand::Eval => {
                // This is handled specially in main.rs
                Ok("Entering eval mode. End with '.' on a line by itself.".to_string())
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
                let player_name = format!("player_{}", name);
                match self.storage.objects.find_by_name(&player_name) {
                    Ok(Some(player_id)) => {
                        self.evaluator.switch_player(player_id)
                            .map_err(|e| ReplError::StorageError(e.to_string()))?;
                        Ok(format!("Switched to player '{}'", name))
                    }
                    Ok(None) => Err(ReplError::ExecutionError(format!("Player '{}' not found", name))),
                    Err(e) => Err(ReplError::StorageError(e.to_string())),
                }
            }
            ReplCommand::ListPlayers => {
                let players = self.storage.objects.list_all()
                    .map_err(|e| ReplError::StorageError(e.to_string()))?;
                
                let mut player_list = Vec::new();
                for id in players {
                    if let Ok(obj) = self.storage.objects.get(id) {
                        if obj.name.starts_with("player_") {
                            let name = obj.name.trim_start_matches("player_");
                            player_list.push(format!("  {} ({})", name, id));
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
                            let name = obj.name.trim_start_matches("player_");
                            Ok(format!("Current player: {} ({})", name, id))
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
}