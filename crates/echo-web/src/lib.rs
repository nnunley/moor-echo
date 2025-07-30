//! # Echo Web
//!
//! Web server and UI components for the Echo programming language.
//!
//! This crate provides:
//! - HTTP/WebSocket server for Echo REPL access
//! - Real-time bidirectional communication
//! - Web-based UI for Echo interactions
//! - Event streaming and UI manipulation
//! - Multi-user support and collaboration features

//#![deny(missing_docs)]  // Temporarily disabled during crate extraction
#![warn(clippy::all)]

pub mod events;
pub mod notifier;
pub mod server;
pub mod web_notifier;

pub use events::{EventData, UiUpdate};
pub use notifier::{WebEvent, WebNotifier};
pub use server::{WebServer, WebServerConfig};
pub use web_notifier::{
    EnvironmentVar, ObjectInfo, StateSnapshot, UiUpdate as UiUpdateCommand,
    WebEvent as WebReplEvent, WebNotifier as WebReplNotifier,
};

/// Web server version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Default web server configuration
impl Default for WebServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 8080,
            static_dir: "./static".into(),
            enable_cors: true,
        }
    }
}
