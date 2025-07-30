use std::collections::HashMap;

use crate::{evaluator::Value, storage::ObjectId};

#[derive(Debug, Clone, PartialEq)]
pub struct EventDefinition {
    pub name: String,
    pub parameters: Vec<String>,
    pub owner_object: ObjectId,
    pub parent_event: Option<String>, // For event inheritance
}

#[derive(Debug, PartialEq)]
pub struct EventHandler {
    pub event_pattern: String, // Can be an event name or a parent event name
    pub condition: Option<crate::ast::EchoAst>,
    pub body: Vec<crate::ast::EchoAst>,
    pub owner_object: ObjectId,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EventInstance {
    pub event_name: String,
    pub arguments: HashMap<String, Value>,
    pub emitter: ObjectId,
    pub timestamp: u64,
}

// Define a base Error event and its descendants for system errors
pub const ERROR_EVENT_NAME: &str = "Error";
// Other specific error events like "PermissionError" will extend "Error"

// This would be a global registry or part of the evaluator
pub struct EventRegistry {
    pub definitions: HashMap<String, EventDefinition>,
    pub handlers: Vec<EventHandler>,
}

impl Default for EventRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl EventRegistry {
    pub fn new() -> Self {
        EventRegistry {
            definitions: HashMap::new(),
            handlers: Vec::new(),
        }
    }

    pub fn register_event_definition(&mut self, def: EventDefinition) {
        self.definitions.insert(def.name.clone(), def);
    }

    pub fn register_event_handler(&mut self, handler: EventHandler) {
        self.handlers.push(handler);
    }

    // Helper to check if an event is a descendant of another
    pub fn is_descendant_event(&self, child_event_name: &str, parent_event_name: &str) -> bool {
        if child_event_name == parent_event_name {
            return true;
        }
        let mut current_event_name = child_event_name.to_string();
        while let Some(def) = self.definitions.get(&current_event_name) {
            if let Some(parent) = &def.parent_event {
                if parent == parent_event_name {
                    return true;
                }
                current_event_name = parent.clone();
            } else {
                break;
            }
        }
        false
    }
}
