use thiserror::Error;
use std::sync::Arc;
use std::path::PathBuf;
use anyhow::Result;

use crate::parser::{EchoParser, EchoAst};
use crate::evaluator::Evaluator;
use crate::storage::Storage;

#[derive(Debug, Clone, PartialEq)]
pub enum ReplCommand {
    Help,
    Quit,
    Execute(String),
    // Player management commands
    CreatePlayer(String),
    SwitchPlayer(String),
    ListPlayers,
    CurrentPlayer,
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
    parser: EchoParser,
    evaluator: Evaluator,
    storage: Arc<Storage>,
}

impl Repl {
    pub fn new() -> Self {
        Self::with_storage_path("./echo-db").expect("Failed to create REPL")
    }
    
    pub fn with_storage_path(path: impl Into<PathBuf>) -> Result<Self> {
        let storage = Arc::new(Storage::new(path.into())?);
        let parser = EchoParser::new()?;
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
    
    pub fn execute(&mut self, code: &str) -> Result<String, ReplError> {
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
        
        // Handle special cases for object creation
        if let EchoAst::ObjectDef { .. } = ast {
            Ok("object created".to_string())
        } else {
            Ok(result.to_string())
        }
    }
    
    pub fn handle_command(&mut self, command: ReplCommand) -> Result<String, ReplError> {
        match command {
            ReplCommand::Help => Ok(self.show_help()),
            ReplCommand::Quit => {
                self.running = false;
                Ok("Goodbye!".to_string())
            }
            ReplCommand::Execute(code) => self.execute(&code),
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
        }
    }
    
    fn show_help(&self) -> String {
        r#"Echo REPL Commands:
  .help                  Show this help message
  .quit                  Exit the REPL
  .player create <name>  Create a new player
  .player switch <name>  Switch to a different player
  .player list          List all players
  .player               Show current player
  
Echo Language:
  let x = 42;           Variable binding
  2 + 2                 Arithmetic expressions
  object name           Object creation
    property p = "v";
  endobject"#.to_string()
    }
}