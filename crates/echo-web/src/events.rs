//! Event system for web UI integration
//!
//! Defines events and data structures for communicating between
//! Echo runtime and web-based user interfaces.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Generic event data from Echo runtime
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventData {
    /// Event pattern/name
    pub pattern: String,
    /// Event payload data
    pub data: serde_json::Value,
    /// Timestamp of the event
    pub timestamp: u64,
    /// Source of the event (player, system, etc.)
    pub source: Option<String>,
}

/// UI update operations
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "operation", content = "params")]
pub enum UiUpdate {
    /// Clear the UI
    Clear,
    /// Add a button element
    AddButton {
        id: String,
        text: String,
        onclick: Option<String>,
    },
    /// Add a text element
    AddText {
        id: String,
        content: String,
        style: Option<String>,
    },
    /// Add a div container
    AddDiv {
        id: String,
        class: Option<String>,
        style: Option<String>,
    },
    /// Add an input field
    AddInput {
        id: String,
        input_type: String,
        placeholder: Option<String>,
        value: Option<String>,
    },
    /// Update an existing element
    Update {
        id: String,
        properties: HashMap<String, String>,
    },
    /// Remove an element
    Remove { id: String },
    /// Set focus to an element
    Focus { id: String },
    /// Show/hide an element
    SetVisible { id: String, visible: bool },
}

impl EventData {
    /// Create a new event
    pub fn new(pattern: String, data: serde_json::Value) -> Self {
        Self {
            pattern,
            data,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            source: None,
        }
    }

    /// Create a new event with source
    pub fn with_source(pattern: String, data: serde_json::Value, source: String) -> Self {
        Self {
            pattern,
            data,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            source: Some(source),
        }
    }
}

impl UiUpdate {
    /// Create a button update
    pub fn button(id: String, text: String) -> Self {
        Self::AddButton {
            id,
            text,
            onclick: None,
        }
    }

    /// Create a button update with click handler
    pub fn button_with_click(id: String, text: String, onclick: String) -> Self {
        Self::AddButton {
            id,
            text,
            onclick: Some(onclick),
        }
    }

    /// Create a text update
    pub fn text(id: String, content: String) -> Self {
        Self::AddText {
            id,
            content,
            style: None,
        }
    }

    /// Create a text update with style
    pub fn text_with_style(id: String, content: String, style: String) -> Self {
        Self::AddText {
            id,
            content,
            style: Some(style),
        }
    }

    /// Create a div update
    pub fn div(id: String) -> Self {
        Self::AddDiv {
            id,
            class: None,
            style: None,
        }
    }

    /// Create a div update with class and style
    pub fn div_with_attrs(id: String, class: Option<String>, style: Option<String>) -> Self {
        Self::AddDiv { id, class, style }
    }

    /// Create an input update
    pub fn input(id: String, input_type: String) -> Self {
        Self::AddInput {
            id,
            input_type,
            placeholder: None,
            value: None,
        }
    }

    /// Create an input update with placeholder and value
    pub fn input_with_attrs(
        id: String,
        input_type: String,
        placeholder: Option<String>,
        value: Option<String>,
    ) -> Self {
        Self::AddInput {
            id,
            input_type,
            placeholder,
            value,
        }
    }

    /// Create an update operation
    pub fn update(id: String, properties: HashMap<String, String>) -> Self {
        Self::Update { id, properties }
    }

    /// Create a remove operation
    pub fn remove(id: String) -> Self {
        Self::Remove { id }
    }

    /// Create a focus operation
    pub fn focus(id: String) -> Self {
        Self::Focus { id }
    }

    /// Create a visibility operation
    pub fn set_visible(id: String, visible: bool) -> Self {
        Self::SetVisible { id, visible }
    }
}
