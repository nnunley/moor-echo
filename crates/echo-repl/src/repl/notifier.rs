//! Output notification system for the REPL
//! 
//! Provides a trait-based system for handling REPL output, allowing different
//! output backends (console, web, file, etc.) to be plugged in.

/// Trait for handling REPL output notifications
pub trait ReplNotifier: Send + Sync {
    /// Handle regular output
    fn on_output(&self, content: &str);
    
    /// Handle error output
    fn on_error(&self, content: &str);
    
    /// Handle evaluation result with timing information
    fn on_result(&self, output: &str, duration_ms: u64, quiet: bool);
}

/// Default console-based notifier
pub struct DefaultNotifier;

impl DefaultNotifier {
    /// Create a new default notifier
    pub fn new() -> Self {
        Self
    }
}

impl ReplNotifier for DefaultNotifier {
    fn on_output(&self, content: &str) {
        if !content.is_empty() {
            println!("{}", content);
        }
    }
    
    fn on_error(&self, content: &str) {
        eprintln!("{}", content);
    }
    
    fn on_result(&self, output: &str, duration_ms: u64, quiet: bool) {
        if output.is_empty() || output == "null" {
            if !quiet {
                println!("=> null ({}ms)", duration_ms);
            }
        } else {
            if quiet {
                println!("{}", output);
            } else {
                println!("=> {} ({}ms)", output, duration_ms);
            }
        }
    }
}

impl Default for DefaultNotifier {
    fn default() -> Self {
        Self::new()
    }
}