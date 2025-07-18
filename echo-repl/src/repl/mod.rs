use thiserror::Error;

#[derive(Debug, Clone, PartialEq)]
pub enum ReplCommand {
    Help,
    Quit,
    Execute(String),
}

#[derive(Error, Debug, PartialEq)]
pub enum ReplError {
    #[error("Parse error: {0}")]
    ParseError(String),
    
    #[error("Execution error: {0}")]
    ExecutionError(String),
    
    #[error("Unknown command: {0}")]
    UnknownCommand(String),
}

pub struct Repl {
    running: bool,
}

impl Repl {
    pub fn new() -> Self {
        Self {
            running: true,
        }
    }
    
    pub fn is_running(&self) -> bool {
        self.running
    }
    
    pub fn parse_input(&self, input: &str) -> Result<ReplCommand, ReplError> {
        let trimmed = input.trim();
        
        if trimmed.starts_with('.') {
            match trimmed {
                ".help" => Ok(ReplCommand::Help),
                ".quit" => Ok(ReplCommand::Quit),
                cmd => Err(ReplError::UnknownCommand(cmd.to_string())),
            }
        } else {
            Ok(ReplCommand::Execute(trimmed.to_string()))
        }
    }
    
    pub fn execute(&mut self, _code: &str) -> Result<String, ReplError> {
        // Placeholder implementation - will be replaced with actual Echo interpreter
        Err(ReplError::ExecutionError("Not implemented".to_string()))
    }
}