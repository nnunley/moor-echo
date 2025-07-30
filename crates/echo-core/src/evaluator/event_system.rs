use std::{collections::HashMap, sync::Arc};

use anyhow::Result;
use parking_lot::RwLock;

use crate::{
    ast::EchoAst,
    evaluator::{Evaluator, Value},
    storage::ObjectId,
};

/// Represents a registered event handler
#[derive(Debug, Clone)]
pub struct EventHandler {
    /// The object that owns this handler
    pub owner: ObjectId,
    /// The event name pattern this handler responds to
    pub event_name: String,
    /// Parameter names for the handler
    pub params: Vec<String>,
    /// The AST body of the handler
    pub body: Vec<EchoAst>,
    /// Priority for handler execution order (higher = earlier)
    pub priority: i32,
}

/// An event instance that can be emitted
#[derive(Debug, Clone)]
pub struct Event {
    /// Name of the event
    pub name: String,
    /// Arguments passed to the event
    pub args: Vec<Value>,
    /// Object that emitted the event
    pub emitter: ObjectId,
    /// Whether this event should bubble up the object hierarchy
    pub bubbles: bool,
    /// Whether handlers can cancel this event
    pub cancelable: bool,
}

/// Result of handling an event
#[derive(Debug, Clone, PartialEq)]
pub enum EventResult {
    /// Event was handled normally
    Handled,
    /// Event was cancelled by a handler
    Cancelled,
    /// Event was not handled by any handler
    Unhandled,
}

/// Type alias for event callback function
pub type EventCallback = Arc<dyn Fn(&Event) -> Result<()> + Send + Sync>;

/// Event subscription for external listeners
#[derive(Clone)]
pub struct EventSubscription {
    pub id: u64,
    pub event_pattern: String,
    pub callback: EventCallback,
}

impl std::fmt::Debug for EventSubscription {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EventSubscription")
            .field("id", &self.id)
            .field("event_pattern", &self.event_pattern)
            .field("callback", &"<function>")
            .finish()
    }
}

/// The main event system registry
pub struct EventSystem {
    /// All registered event handlers by event name
    handlers: RwLock<HashMap<String, Vec<EventHandler>>>,
    /// Event subscriptions for external listeners
    subscriptions: RwLock<HashMap<u64, EventSubscription>>,
    /// Next subscription ID
    next_subscription_id: RwLock<u64>,
    /// Global event handlers (match any event)
    global_handlers: RwLock<Vec<EventHandler>>,
}

impl EventSystem {
    pub fn new() -> Self {
        EventSystem {
            handlers: RwLock::new(HashMap::new()),
            subscriptions: RwLock::new(HashMap::new()),
            next_subscription_id: RwLock::new(1),
            global_handlers: RwLock::new(Vec::new()),
        }
    }

    /// Register an event handler from an object
    pub fn register_handler(
        &self,
        owner: ObjectId,
        event_name: String,
        params: Vec<String>,
        body: Vec<EchoAst>,
        priority: Option<i32>,
    ) {
        let handler = EventHandler {
            owner,
            event_name: event_name.clone(),
            params,
            body,
            priority: priority.unwrap_or(0),
        };

        let mut handlers = self.handlers.write();
        handlers
            .entry(event_name.clone())
            .or_default()
            .push(handler);

        // Sort by priority (descending)
        if let Some(handler_list) = handlers.get_mut(&event_name) {
            handler_list.sort_by(|a, b| b.priority.cmp(&a.priority));
        }
    }

    /// Register a global handler that receives all events
    pub fn register_global_handler(
        &self,
        owner: ObjectId,
        params: Vec<String>,
        body: Vec<EchoAst>,
        priority: Option<i32>,
    ) {
        let handler = EventHandler {
            owner,
            event_name: "*".to_string(),
            params,
            body,
            priority: priority.unwrap_or(0),
        };

        let mut global_handlers = self.global_handlers.write();
        global_handlers.push(handler);
        global_handlers.sort_by(|a, b| b.priority.cmp(&a.priority));
    }

    /// Emit an event and execute all matching handlers
    pub fn emit(&self, evaluator: &mut Evaluator, event: Event) -> Result<EventResult> {
        let handlers = self.handlers.read();
        let global_handlers = self.global_handlers.read();

        // Collect all matching handlers
        let mut all_handlers = Vec::new();

        // Add specific handlers for this event
        if let Some(specific_handlers) = handlers.get(&event.name) {
            all_handlers.extend(specific_handlers.iter());
        }

        // Add wildcard handlers
        if let Some(wildcard_handlers) = handlers.get("*") {
            all_handlers.extend(wildcard_handlers.iter());
        }

        // Add global handlers
        all_handlers.extend(global_handlers.iter());

        // Sort by priority again (in case we mixed different lists)
        all_handlers.sort_by(|a, b| b.priority.cmp(&a.priority));

        // Execute handlers
        let mut handled = false;
        for handler in all_handlers {
            match self.execute_handler(evaluator, handler, &event)? {
                EventResult::Cancelled => return Ok(EventResult::Cancelled),
                EventResult::Handled => handled = true,
                EventResult::Unhandled => {}
            }
        }

        // Notify subscriptions
        self.notify_subscriptions(&event)?;

        Ok(if handled {
            EventResult::Handled
        } else {
            EventResult::Unhandled
        })
    }

    /// Execute a single event handler
    fn execute_handler(
        &self,
        evaluator: &mut Evaluator,
        handler: &EventHandler,
        event: &Event,
    ) -> Result<EventResult> {
        // Create a new environment for the handler
        let handler_env = evaluator.create_handler_environment(handler.owner);

        // Bind event arguments to parameters
        for (i, param_name) in handler.params.iter().enumerate() {
            let value = event.args.get(i).cloned().unwrap_or(Value::Null);
            evaluator.set_variable_in_env(&handler_env, param_name, value)?;
        }

        // Bind special event variables
        evaluator.set_variable_in_env(
            &handler_env,
            "$event_name",
            Value::String(event.name.clone()),
        )?;
        evaluator.set_variable_in_env(
            &handler_env,
            "$event_emitter",
            Value::Object(event.emitter),
        )?;

        // Execute handler body
        let prev_env = evaluator.push_environment(handler_env);
        let result = EventResult::Handled;

        for stmt in &handler.body {
            let _value = evaluator.eval_with_player(stmt, handler.owner)?;
            // Check if handler returned false to cancel event
            // Note: Since we're using eval_with_player which returns Value,
            // we need to handle return statements differently
            // For now, we'll just execute all statements
            // TODO: Add support for early return from event handlers
        }

        evaluator.pop_environment(prev_env);
        Ok(result)
    }

    /// Subscribe to events with a callback
    pub fn subscribe<F>(&self, event_pattern: String, callback: F) -> u64
    where
        F: Fn(&Event) -> Result<()> + Send + Sync + 'static,
    {
        let mut next_id = self.next_subscription_id.write();
        let id = *next_id;
        *next_id += 1;

        let subscription = EventSubscription {
            id,
            event_pattern,
            callback: Arc::new(callback),
        };

        let mut subscriptions = self.subscriptions.write();
        subscriptions.insert(id, subscription);

        id
    }

    /// Unsubscribe from events
    pub fn unsubscribe(&self, subscription_id: u64) -> bool {
        let mut subscriptions = self.subscriptions.write();
        subscriptions.remove(&subscription_id).is_some()
    }

    /// Notify all subscriptions of an event
    fn notify_subscriptions(&self, event: &Event) -> Result<()> {
        let subscriptions = self.subscriptions.read();

        for subscription in subscriptions.values() {
            if self.matches_pattern(&event.name, &subscription.event_pattern) {
                (subscription.callback)(event)?;
            }
        }

        Ok(())
    }

    /// Check if an event name matches a pattern
    fn matches_pattern(&self, event_name: &str, pattern: &str) -> bool {
        if pattern == "*" {
            return true;
        }

        // Simple glob matching
        if let Some(prefix) = pattern.strip_suffix('*') {
            event_name.starts_with(prefix)
        } else {
            event_name == pattern
        }
    }

    /// Remove all handlers for a specific object
    pub fn remove_object_handlers(&self, object_id: ObjectId) {
        let mut handlers = self.handlers.write();
        for handler_list in handlers.values_mut() {
            handler_list.retain(|h| h.owner != object_id);
        }

        let mut global_handlers = self.global_handlers.write();
        global_handlers.retain(|h| h.owner != object_id);
    }

    /// Get all registered event names
    pub fn get_event_names(&self) -> Vec<String> {
        let handlers = self.handlers.read();
        handlers.keys().cloned().collect()
    }

    /// Get handler count for debugging
    pub fn handler_count(&self) -> usize {
        let handlers = self.handlers.read();
        let global_handlers = self.global_handlers.read();

        handlers.values().map(|v| v.len()).sum::<usize>() + global_handlers.len()
    }
}

impl Default for EventSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_pattern_matching() {
        let system = EventSystem::new();

        assert!(system.matches_pattern("player_moved", "player_moved"));
        assert!(system.matches_pattern("player_moved", "*"));
        assert!(system.matches_pattern("player_moved", "player_*"));
        assert!(!system.matches_pattern("player_moved", "enemy_*"));
        assert!(!system.matches_pattern("player_moved", "player"));
    }
}
