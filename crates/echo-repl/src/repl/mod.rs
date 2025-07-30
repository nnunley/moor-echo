//! REPL (Read-Eval-Print Loop) functionality for Echo
//!
//! This module provides interactive command-line interface components that wrap
//! the core Echo runtime with user-friendly features like:
//! - Command history and editing
//! - Multi-line input collection
//! - REPL commands (.help, .quit, etc.)
//! - Player management
//! - Output formatting and notifications

use std::time::Instant;

use anyhow::Result;
use echo_core::{EchoRuntime, Parser, Value};

pub mod commands;
pub mod multiline;
pub mod notifier;

pub use commands::ReplCommand;
pub use multiline::{LineProcessResult, MultiLineCollector};
pub use notifier::{DefaultNotifier, ReplNotifier};

/// Interactive REPL for the Echo programming language
pub struct Repl {
    /// Core Echo runtime
    runtime: EchoRuntime,
    /// Current notifier for output
    notifier: Box<dyn ReplNotifier>,
    /// Whether the REPL is running
    running: bool,
    /// Quiet mode (suppress timing info)
    quiet: bool,
    /// Debug mode
    debug: bool,
}

impl Repl {
    /// Create a new REPL with the given runtime
    pub fn new(runtime: EchoRuntime) -> Result<Self> {
        Ok(Self {
            runtime,
            notifier: Box::new(DefaultNotifier::new()),
            running: true,
            quiet: false,
            debug: false,
        })
    }

    /// Create a new REPL with a specific storage path (for testing)
    pub fn with_storage_path<P: Into<std::path::PathBuf>>(path: P) -> Result<Self> {
        use echo_core::{EchoConfig, EchoRuntime};

        let config = EchoConfig {
            storage_path: path.into(),
            ..Default::default()
        };
        let runtime = EchoRuntime::new(config)?;
        Self::new(runtime)
    }

    /// Set the notifier for this REPL
    pub fn set_notifier(&mut self, notifier: Box<dyn ReplNotifier>) {
        self.notifier = notifier;
    }

    /// Get a reference to the current notifier
    pub fn notifier(&self) -> &dyn ReplNotifier {
        self.notifier.as_ref()
    }

    /// Get a mutable reference to the parser
    pub fn parser_mut(&mut self) -> &mut Box<dyn Parser> {
        self.runtime.parser_mut()
    }

    /// Check if the REPL is still running
    pub fn is_running(&self) -> bool {
        self.running
    }

    /// Check if quiet mode is enabled
    pub fn is_quiet(&self) -> bool {
        self.quiet
    }

    /// Set quiet mode
    pub fn set_quiet(&mut self, quiet: bool) {
        self.quiet = quiet;
    }

    /// Set debug mode
    pub fn set_debug(&mut self, debug: bool) {
        self.debug = debug;
    }

    /// Parse REPL input into a command
    pub fn parse_input(&self, input: &str) -> Result<ReplCommand> {
        commands::parse_command(input)
    }

    /// Handle a REPL command
    pub fn handle_command(&mut self, command: ReplCommand) -> Result<String> {
        match command {
            ReplCommand::Help => Ok(self.get_help_text()),
            ReplCommand::Quit => {
                self.running = false;
                Ok("Goodbye!".to_string())
            }
            ReplCommand::Clear => {
                print!("\x1B[2J\x1B[1;1H");
                Ok("Screen cleared.".to_string())
            }
            ReplCommand::Quiet => {
                self.quiet = !self.quiet;
                Ok(format!(
                    "Quiet mode: {}",
                    if self.quiet { "on" } else { "off" }
                ))
            }
            ReplCommand::Debug => {
                self.debug = !self.debug;
                Ok(format!(
                    "Debug mode: {}",
                    if self.debug { "on" } else { "off" }
                ))
            }
            ReplCommand::CreatePlayer(name) => self.create_player(&name),
            ReplCommand::SwitchPlayer(name) => self.switch_player(&name),
            ReplCommand::ListPlayers => self.list_players(),
            ReplCommand::Stats => self.show_stats(),
        }
    }

    /// Execute Echo code and return the result with timing
    pub fn execute(&mut self, code: &str) -> Result<(String, u64)> {
        let start = Instant::now();

        match self.runtime.eval_source(code) {
            Ok(value) => {
                let duration = start.elapsed().as_millis() as u64;
                let output = self.format_value(&value);
                Ok((output, duration))
            }
            Err(e) => Err(e.into()),
        }
    }

    /// Execute Echo program (multi-statement) and return the result with timing
    pub fn execute_program(&mut self, code: &str) -> Result<(String, u64)> {
        let start = Instant::now();

        match self.runtime.parse_program(code) {
            Ok(ast) => match self.runtime.eval(&ast) {
                Ok(value) => {
                    let duration = start.elapsed().as_millis() as u64;
                    let output = self.format_value(&value);
                    Ok((output, duration))
                }
                Err(e) => Err(e.into()),
            },
            Err(e) => Err(e.into()),
        }
    }

    /// Format a value for display
    fn format_value(&self, value: &Value) -> String {
        match value {
            Value::Null => "null".to_string(),
            Value::Boolean(b) => b.to_string(),
            Value::Integer(i) => i.to_string(),
            Value::Float(f) => f.to_string(),
            Value::String(s) => format!("\"{}\"", s),
            Value::List(items) => {
                let formatted: Vec<String> = items.iter().map(|v| self.format_value(v)).collect();
                format!("[{}]", formatted.join(", "))
            }
            Value::Object(id) => format!("#{}", id),
            Value::Map(map) => {
                let formatted: Vec<String> = map
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, self.format_value(v)))
                    .collect();
                format!("{{{}}}", formatted.join(", "))
            }
            Value::Lambda { .. } => "<lambda>".to_string(),
        }
    }

    /// Get help text
    fn get_help_text(&self) -> String {
        r#"Echo REPL Commands:
  .help    - Show this help message
  .quit    - Exit the REPL
  .clear   - Clear the screen
  .quiet   - Toggle quiet mode (hide timing info)
  .debug   - Toggle debug mode
  .eval    - Enter multi-line evaluation mode (end with '.')
  
Player Commands:
  .create <name>  - Create a new player
  .switch <name>  - Switch to a player
  .players        - List all players
  .stats          - Show runtime statistics

Echo Language Features:
  - Variables: let x = 42
  - Lists: {1, 2, 3}
  - Arrow Functions: x => x * 2
  - Block Functions: fn {x} x * 2 endfn
  - Conditionals: if (condition) ... else ... endif
  - Loops: for x in ({1, 2, 3}) ... endfor
  - Objects: object #123 property name = "test" endobject
  - Events: emit event_name({data})

Type 'help' for language help, or visit the documentation."#
            .to_string()
    }

    /// Create a new player
    fn create_player(&mut self, name: &str) -> Result<String> {
        // Use the runtime to create a player directly
        match self.runtime.create_player(name) {
            Ok(player_id) => {
                // Auto-switch to the newly created player
                if let Err(e) = self.runtime.switch_player(player_id) {
                    return Err(anyhow::anyhow!(
                        "Created player '{}' but failed to switch: {}",
                        name,
                        e
                    ));
                }
                Ok(format!("Created and switched to player '{}'", name))
            }
            Err(e) => {
                if e.to_string().contains("already exists") {
                    Err(anyhow::anyhow!("Player '{}' already exists", name))
                } else {
                    Err(e.into())
                }
            }
        }
    }

    /// Switch to a player
    fn switch_player(&mut self, name: &str) -> Result<String> {
        match self.runtime.switch_player_by_name(name) {
            Ok(_) => Ok(format!("Switched to player '{}'", name)),
            Err(e) => Err(e.into()),
        }
    }

    /// List all players
    fn list_players(&mut self) -> Result<String> {
        match self.runtime.list_players() {
            Ok(players) => {
                if players.is_empty() {
                    Ok("No players found.".to_string())
                } else {
                    let player_list = players
                        .iter()
                        .map(|(name, id)| format!("  {} (#{:x})", name, id.0))
                        .collect::<Vec<_>>()
                        .join("\n");
                    Ok(format!("Players:\n{}", player_list))
                }
            }
            Err(e) => Err(e.into()),
        }
    }

    /// Show runtime statistics
    fn show_stats(&self) -> Result<String> {
        let stats = format!(
            "Runtime Statistics:\n  Objects in storage: {}\n  Debug mode: {}\n  Quiet mode: {}",
            "N/A", // We'd need to expose this from storage
            self.debug,
            self.quiet
        );
        Ok(stats)
    }

    /// Show exit statistics
    pub fn show_exit_stats(&self) {
        if !self.quiet {
            println!("\nSession complete.");
        }
    }
}
