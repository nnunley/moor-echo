//! REPL command parsing and definitions
//!
//! Handles parsing of dot-commands (.help, .quit, etc.) and player management
//! commands.

use anyhow::{anyhow, Result};

/// Available REPL commands
#[derive(Debug, Clone)]
pub enum ReplCommand {
    /// Show help information
    Help,
    /// Exit the REPL
    Quit,
    /// Clear the screen
    Clear,
    /// Toggle quiet mode
    Quiet,
    /// Toggle debug mode
    Debug,
    /// Create a new player
    CreatePlayer(String),
    /// Switch to an existing player
    SwitchPlayer(String),
    /// List all players
    ListPlayers,
    /// Show runtime statistics
    Stats,
    /// Import MOO file
    ImportMoo(String),
}

/// Parse a command string into a ReplCommand
pub fn parse_command(input: &str) -> Result<ReplCommand> {
    let trimmed = input.trim();

    if !trimmed.starts_with('.') {
        return Err(anyhow!("Commands must start with '.'"));
    }

    let parts: Vec<&str> = trimmed[1..].split_whitespace().collect();

    if parts.is_empty() {
        return Err(anyhow!("Empty command"));
    }

    match parts[0] {
        "help" | "h" => Ok(ReplCommand::Help),
        "quit" | "q" | "exit" => Ok(ReplCommand::Quit),
        "clear" | "cls" => Ok(ReplCommand::Clear),
        "quiet" => Ok(ReplCommand::Quiet),
        "debug" => Ok(ReplCommand::Debug),
        "create" => {
            if parts.len() != 2 {
                return Err(anyhow!("Usage: .create <player_name>"));
            }
            Ok(ReplCommand::CreatePlayer(parts[1].to_string()))
        }
        "switch" => {
            if parts.len() != 2 {
                return Err(anyhow!("Usage: .switch <player_name>"));
            }
            Ok(ReplCommand::SwitchPlayer(parts[1].to_string()))
        }
        "players" | "list" => Ok(ReplCommand::ListPlayers),
        "stats" | "statistics" => Ok(ReplCommand::Stats),
        "import" => {
            if parts.len() != 2 {
                return Err(anyhow!("Usage: .import <moo_file>"));
            }
            Ok(ReplCommand::ImportMoo(parts[1].to_string()))
        }
        _ => Err(anyhow!("Unknown command: .{}", parts[0])),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_help() {
        assert!(matches!(parse_command(".help").unwrap(), ReplCommand::Help));
        assert!(matches!(parse_command(".h").unwrap(), ReplCommand::Help));
    }

    #[test]
    fn test_parse_quit() {
        assert!(matches!(parse_command(".quit").unwrap(), ReplCommand::Quit));
        assert!(matches!(parse_command(".q").unwrap(), ReplCommand::Quit));
        assert!(matches!(parse_command(".exit").unwrap(), ReplCommand::Quit));
    }

    #[test]
    fn test_parse_create_player() {
        match parse_command(".create alice").unwrap() {
            ReplCommand::CreatePlayer(name) => assert_eq!(name, "alice"),
            _ => panic!("Expected CreatePlayer command"),
        }
    }

    #[test]
    fn test_parse_switch_player() {
        match parse_command(".switch bob").unwrap() {
            ReplCommand::SwitchPlayer(name) => assert_eq!(name, "bob"),
            _ => panic!("Expected SwitchPlayer command"),
        }
    }

    #[test]
    fn test_parse_invalid_command() {
        assert!(parse_command(".invalid").is_err());
        assert!(parse_command("help").is_err()); // Missing dot
        assert!(parse_command(".create").is_err()); // Missing argument
    }
}
