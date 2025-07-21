use std::sync::Arc;
use serde_json::json;

/// Trait for handling UI updates from the evaluator
pub trait UiHandler: Send + Sync {
    /// Clear the dynamic UI area
    fn ui_clear(&self);
    
    /// Add a button to the dynamic UI
    fn ui_add_button(&self, id: &str, label: &str, action: &str);
    
    /// Add text to the dynamic UI
    fn ui_add_text(&self, id: &str, text: &str, style: Option<serde_json::Value>);
    
    /// Update an existing UI element
    fn ui_update(&self, id: &str, properties: serde_json::Value);
    
    /// Add a divider
    fn ui_add_divider(&self, id: &str);
}

/// Default implementation that does nothing (for non-web contexts)
pub struct NoOpUiHandler;

impl UiHandler for NoOpUiHandler {
    fn ui_clear(&self) {}
    fn ui_add_button(&self, _id: &str, _label: &str, _action: &str) {}
    fn ui_add_text(&self, _id: &str, _text: &str, _style: Option<serde_json::Value>) {}
    fn ui_update(&self, _id: &str, _properties: serde_json::Value) {}
    fn ui_add_divider(&self, _id: &str) {}
}

/// Implementation that sends updates through WebNotifier
#[cfg(feature = "web-ui")]
pub struct WebUiHandler {
    notifier: Arc<crate::repl::web_notifier::WebNotifier>,
}

#[cfg(feature = "web-ui")]
impl WebUiHandler {
    pub fn new(notifier: Arc<crate::repl::web_notifier::WebNotifier>) -> Self {
        Self { notifier }
    }
}

#[cfg(feature = "web-ui")]
impl UiHandler for WebUiHandler {
    fn ui_clear(&self) {
        use crate::repl::web_notifier::UiUpdate;
        
        self.notifier.send_ui_update(UiUpdate {
            target: "dynamicContent".to_string(),
            action: "clear".to_string(),
            data: json!({}),
        });
    }
    
    fn ui_add_button(&self, id: &str, label: &str, action: &str) {
        use crate::repl::web_notifier::UiUpdate;
        
        self.notifier.send_ui_update(UiUpdate {
            target: id.to_string(),
            action: "add_button".to_string(),
            data: json!({
                "label": label,
                "action": action,
            }),
        });
    }
    
    fn ui_add_text(&self, id: &str, text: &str, style: Option<serde_json::Value>) {
        use crate::repl::web_notifier::UiUpdate;
        
        let mut data = json!({
            "text": text,
        });
        
        if let Some(style) = style {
            data["style"] = style;
        }
        
        self.notifier.send_ui_update(UiUpdate {
            target: id.to_string(),
            action: "add_text".to_string(),
            data,
        });
    }
    
    fn ui_update(&self, id: &str, properties: serde_json::Value) {
        use crate::repl::web_notifier::UiUpdate;
        
        self.notifier.send_ui_update(UiUpdate {
            target: id.to_string(),
            action: "update".to_string(),
            data: json!({
                "properties": properties,
            }),
        });
    }
    
    fn ui_add_divider(&self, id: &str) {
        use crate::repl::web_notifier::UiUpdate;
        
        self.notifier.send_ui_update(UiUpdate {
            target: id.to_string(),
            action: "add_divider".to_string(),
            data: json!({}),
        });
    }
}