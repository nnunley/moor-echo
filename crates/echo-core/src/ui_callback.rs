//! UI callback system for external UI event handling
//!
//! This module provides a callback mechanism that allows external components
//! (like web servers) to receive UI events emitted by the Echo evaluator.

use std::collections::HashMap;

use crate::evaluator::Value;

// Helper function to extract style from args
fn extract_style_from_args(args: &[Value], index: usize) -> Option<HashMap<String, String>> {
    if args.len() > index {
        match &args[index] {
            Value::Map(map) => {
                let mut style_map = HashMap::new();
                for (k, v) in map {
                    style_map.insert(k.clone(), v.to_string());
                }
                Some(style_map)
            }
            _ => None,
        }
    } else {
        None
    }
}

/// UI update action types
#[derive(Debug, Clone)]
pub enum UiAction {
    Clear,
    AddButton {
        id: String,
        label: String,
        action: String,
    },
    AddText {
        id: String,
        text: String,
        style: Option<HashMap<String, String>>,
    },
    AddDiv {
        id: String,
        content: String,
        style: Option<HashMap<String, String>>,
    },
    Update {
        id: String,
        properties: HashMap<String, Value>,
    },
    /// Send a notification message to a player
    NotifyPlayer {
        player_id: crate::storage::ObjectId,
        message: String,
    },
}

/// UI event that can be sent to external handlers
#[derive(Debug, Clone)]
pub struct UiEvent {
    /// The UI action to perform
    pub action: UiAction,
    /// Target element ID (for actions that target specific elements)
    pub target: String,
    /// Additional data for the action
    pub data: HashMap<String, Value>,
}

/// Trait for UI event handlers
pub trait UiEventHandler: Send + Sync {
    /// Handle a UI event
    fn handle_ui_event(&self, event: UiEvent);
}

/// Type alias for UI event callback functions
pub type UiEventCallback = std::sync::Arc<dyn Fn(UiEvent) + Send + Sync>;

/// Convert evaluator UI event arguments to UiAction
pub fn convert_ui_event(action: &str, args: &[Value]) -> Option<UiAction> {
    match action {
        "clear" => Some(UiAction::Clear),
        "add_button" => {
            if args.len() >= 3 {
                if let (Value::String(id), Value::String(label), Value::String(action)) =
                    (&args[0], &args[1], &args[2])
                {
                    return Some(UiAction::AddButton {
                        id: id.clone(),
                        label: label.clone(),
                        action: action.clone(),
                    });
                }
            }
            None
        }
        "add_text" => {
            if args.len() >= 2 {
                if let (Value::String(id), Value::String(text)) = (&args[0], &args[1]) {
                    let style = extract_style_from_args(args, 2);

                    return Some(UiAction::AddText {
                        id: id.clone(),
                        text: text.clone(),
                        style,
                    });
                }
            }
            None
        }
        "add_div" => {
            if args.len() >= 2 {
                if let (Value::String(id), Value::String(content)) = (&args[0], &args[1]) {
                    let style = extract_style_from_args(args, 2);

                    return Some(UiAction::AddDiv {
                        id: id.clone(),
                        content: content.clone(),
                        style,
                    });
                }
            }
            None
        }
        _ => None,
    }
}
