//! Web-based notifier implementation
//!
//! Provides a REPL notifier that streams events to connected web clients
//! via WebSocket or server-sent events.

use echo_core::Value;
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;

use crate::events::{EventData, UiUpdate};

/// Events sent to web clients
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum WebEvent {
    /// Regular output message
    Output { content: String },
    /// Error message
    Error { content: String },
    /// Execution result with timing
    Result { output: String, duration_ms: u64 },
    /// Runtime state update
    StateUpdate { snapshot: StateSnapshot },
    /// UI update event
    UiUpdate { update: UiUpdate },
    /// Generic event from Echo runtime
    Event { event: EventData },
}

/// Snapshot of runtime state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateSnapshot {
    /// Current player information
    pub current_player: Option<String>,
    /// Runtime statistics
    pub stats: RuntimeStats,
}

/// Runtime statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeStats {
    /// Number of objects in storage
    pub object_count: usize,
    /// Evaluation count
    pub eval_count: u64,
    /// Uptime in seconds
    pub uptime_seconds: u64,
}

/// Web-based notifier that streams to connected clients
pub struct WebNotifier {
    /// Broadcast sender for web events
    tx: broadcast::Sender<WebEvent>,
}

impl WebNotifier {
    /// Create a new web notifier
    pub fn new(tx: broadcast::Sender<WebEvent>) -> Self {
        Self { tx }
    }

    /// Send a web event to all connected clients
    pub fn send_event(&self, event: WebEvent) {
        // Ignore send errors (no connected clients)
        let _ = self.tx.send(event);
    }

    /// Create a receiver for web events
    pub fn subscribe(&self) -> broadcast::Receiver<WebEvent> {
        self.tx.subscribe()
    }

    /// Send a UI update event
    pub fn send_ui_update(&self, update: UiUpdate) {
        self.send_event(WebEvent::UiUpdate { update });
    }

    /// Send a generic event from Echo runtime
    pub fn send_echo_event(&self, event: EventData) {
        self.send_event(WebEvent::Event { event });
    }

    /// Send a state update
    pub fn send_state_update(&self, snapshot: StateSnapshot) {
        self.send_event(WebEvent::StateUpdate { snapshot });
    }
}

// Implementation of ReplNotifier trait from echo-repl
// Note: This would require echo-repl as a dependency, or we could define
// a similar trait in echo-core for better decoupling

/// Format a value for web display
pub fn format_web_value(value: &Value) -> String {
    match value {
        Value::Null => "null".to_string(),
        Value::Boolean(b) => b.to_string(),
        Value::Integer(i) => i.to_string(),
        Value::Float(f) => f.to_string(),
        Value::String(s) => format!("\"{s}\""),
        Value::List(items) => {
            let formatted: Vec<String> = items.iter().map(format_web_value).collect();
            format!("[{}]", formatted.join(", "))
        }
        Value::Object(id) => format!("#{id}"),
        Value::Map(map) => {
            let formatted: Vec<String> = map
                .iter()
                .map(|(k, v)| format!("{}: {}", k, format_web_value(v)))
                .collect();
            format!("{{{}}}", formatted.join(", "))
        }
        Value::Lambda { .. } => "<lambda>".to_string(),
    }
}
