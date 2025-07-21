use std::sync::{Arc, Mutex};
use std::collections::VecDeque;
use std::time::Duration;
use tokio::sync::mpsc::{UnboundedSender, UnboundedReceiver, unbounded_channel};
use serde::{Serialize, Deserialize};
use super::ReplNotifier;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum WebEvent {
    Output { content: String },
    Error { content: String },
    Result { output: String, duration_ms: u64 },
    StateUpdate { snapshot: StateSnapshot },
    UiUpdate { update: UiUpdate },
    Event { event: EventData },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventData {
    pub name: String,
    pub args: Vec<serde_json::Value>,
    pub emitter: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateSnapshot {
    pub environment: Vec<EnvironmentVar>,
    pub objects: Vec<ObjectInfo>,
    pub current_player: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentVar {
    pub name: String,
    pub value: String,
    pub var_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectInfo {
    pub id: String,
    pub name: String,
    pub properties: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiUpdate {
    pub target: String,
    pub action: String,
    pub data: serde_json::Value,
}

pub struct WebNotifier {
    sender: Arc<Mutex<Vec<UnboundedSender<WebEvent>>>>,
    buffer: Arc<Mutex<VecDeque<WebEvent>>>,
    buffer_size: usize,
}

impl WebNotifier {
    pub fn new(buffer_size: usize) -> Self {
        Self {
            sender: Arc::new(Mutex::new(Vec::new())),
            buffer: Arc::new(Mutex::new(VecDeque::with_capacity(buffer_size))),
            buffer_size,
        }
    }

    pub fn add_client(&self, sender: UnboundedSender<WebEvent>) {
        let mut senders = self.sender.lock().unwrap();
        senders.push(sender);
        
        // Send buffered events to new client
        let buffer = self.buffer.lock().unwrap();
        for event in buffer.iter() {
            let _ = senders.last().unwrap().send(event.clone());
        }
    }

    pub fn remove_client(&self, sender: &UnboundedSender<WebEvent>) {
        let mut senders = self.sender.lock().unwrap();
        senders.retain(|s| !s.same_channel(sender));
    }

    fn send_event(&self, event: WebEvent) {
        // Add to buffer
        let mut buffer = self.buffer.lock().unwrap();
        if buffer.len() >= self.buffer_size {
            buffer.pop_front();
        }
        buffer.push_back(event.clone());
        drop(buffer);

        // Send to all connected clients
        let mut senders = self.sender.lock().unwrap();
        senders.retain(|sender| {
            sender.send(event.clone()).is_ok()
        });
    }

    pub fn send_state_update(&self, snapshot: StateSnapshot) {
        self.send_event(WebEvent::StateUpdate { snapshot });
    }

    pub fn send_ui_update(&self, update: UiUpdate) {
        self.send_event(WebEvent::UiUpdate { update });
    }
    
    pub fn send_echo_event(&self, event: EventData) {
        self.send_event(WebEvent::Event { event });
    }
}

impl ReplNotifier for WebNotifier {
    fn on_output(&self, output: &str) {
        self.send_event(WebEvent::Output {
            content: output.to_string(),
        });
    }

    fn on_error(&self, error: &str) {
        self.send_event(WebEvent::Error {
            content: error.to_string(),
        });
    }

    fn on_result(&self, output: &str, duration: Duration, quiet: bool) {
        if !quiet {
            self.send_event(WebEvent::Result {
                output: output.to_string(),
                duration_ms: duration.as_millis() as u64,
            });
        }
    }
}

// Client handle for managing WebSocket connections
pub struct WebClient {
    sender: UnboundedSender<WebEvent>,
    notifier: Arc<WebNotifier>,
}

impl WebClient {
    pub fn new(notifier: Arc<WebNotifier>) -> (Self, UnboundedReceiver<WebEvent>) {
        let (sender, receiver) = unbounded_channel();
        notifier.add_client(sender.clone());
        
        let client = Self {
            sender: sender.clone(),
            notifier,
        };
        
        (client, receiver)
    }
}

impl Drop for WebClient {
    fn drop(&mut self) {
        self.notifier.remove_client(&self.sender);
    }
}