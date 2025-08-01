//! Web notifier for streaming REPL output to web clients
//!
//! This module provides WebNotifier which implements ReplNotifier to stream
//! REPL events to connected web clients via WebSocket.

use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
    time::Duration,
};

use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;

/// Events that can be sent to web clients
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum WebEvent {
    /// REPL output message
    Output { content: String },
    /// Error message
    Error { content: String },
    /// Execution result with timing
    Result { output: String, duration_ms: u64 },
    /// State update (environment, objects, etc.)
    StateUpdate { snapshot: StateSnapshot },
    /// Dynamic UI update from REPL
    UiUpdate { update: UiUpdate },
    /// Chat message from a player
    ChatMessage { player: String, message: String },
}

/// Snapshot of current REPL state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateSnapshot {
    /// Environment variables
    pub environment: Vec<EnvironmentVar>,
    /// Objects in the database
    pub objects: Vec<ObjectInfo>,
    /// Current player name
    pub current_player: String,
}

/// Environment variable information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentVar {
    /// Variable name
    pub name: String,
    /// Variable value (as string)
    pub value: String,
    /// Variable type
    pub var_type: String,
}

/// Object information for display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectInfo {
    /// Object ID
    pub id: String,
    /// Object name
    pub name: String,
    /// List of properties
    pub properties: Vec<String>,
}

/// UI update command from REPL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiUpdate {
    /// Target element ID
    pub target: String,
    /// Action to perform (add_button, add_text, clear, etc.)
    pub action: String,
    /// Action data
    pub data: serde_json::Value,
}

/// Web notifier that broadcasts events to connected clients
pub struct WebNotifier {
    /// Broadcast sender for events
    sender: broadcast::Sender<WebEvent>,
    /// Buffer for recent events (for new clients)
    buffer: Arc<Mutex<VecDeque<WebEvent>>>,
    /// Maximum buffer size
    buffer_size: usize,
}

impl WebNotifier {
    /// Create a new WebNotifier with the specified buffer size
    pub fn new(buffer_size: usize) -> Self {
        let (sender, _) = broadcast::channel(1000);

        Self {
            sender,
            buffer: Arc::new(Mutex::new(VecDeque::with_capacity(buffer_size))),
            buffer_size,
        }
    }

    /// Get a receiver for web events
    pub fn subscribe(&self) -> broadcast::Receiver<WebEvent> {
        self.sender.subscribe()
    }

    /// Send buffered events to a new receiver
    pub fn send_buffered_events(&self, _receiver: &broadcast::Receiver<WebEvent>) -> Vec<WebEvent> {
        let buffer = self.buffer.lock().unwrap();
        buffer.iter().cloned().collect()
    }

    /// Send an event to all connected clients
    fn send_event(&self, event: WebEvent) {
        // Add to buffer
        {
            let mut buffer = self.buffer.lock().unwrap();
            if buffer.len() >= self.buffer_size {
                buffer.pop_front();
            }
            buffer.push_back(event.clone());
        }

        // Broadcast to all connected clients
        let _ = self.sender.send(event);
    }

    /// Send a state update event
    pub fn send_state_update(&self, snapshot: StateSnapshot) {
        self.send_event(WebEvent::StateUpdate { snapshot });
    }

    /// Send a UI update event
    pub fn send_ui_update(&self, update: UiUpdate) {
        self.send_event(WebEvent::UiUpdate { update });
    }

    /// Send output message
    pub fn send_output(&self, content: &str) {
        self.send_event(WebEvent::Output {
            content: content.to_string(),
        });
    }

    /// Send error message
    pub fn send_error(&self, content: &str) {
        self.send_event(WebEvent::Error {
            content: content.to_string(),
        });
    }

    /// Send result with timing
    pub fn send_result(&self, output: &str, duration: Duration) {
        self.send_event(WebEvent::Result {
            output: output.to_string(),
            duration_ms: duration.as_millis() as u64,
        });
    }

    /// Send a chat message
    pub fn send_chat_message(&self, player: &str, message: &str) {
        self.send_event(WebEvent::ChatMessage {
            player: player.to_string(),
            message: message.to_string(),
        });
    }
    
    /// Send a player notification (equivalent to MOO's notify())
    pub fn send_player_notification(&self, player_id: &str, message: &str) {
        // For now, treat player notifications as chat messages with system prefix
        self.send_event(WebEvent::ChatMessage {
            player: format!("system@{}", player_id),
            message: message.to_string(),
        });
    }
}

impl Default for WebNotifier {
    fn default() -> Self {
        Self::new(100)
    }
}
