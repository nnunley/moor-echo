use std::sync::Arc;

use anyhow::Result;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use uuid::Uuid;

use crate::storage::ObjectId;

/// Event ID for tracking and ordering
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EventId(pub Uuid);

impl EventId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

/// Echo event representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EchoEvent {
    pub id: EventId,
    pub name: String,
    pub source: ObjectId,
    pub timestamp: u64,
    pub args: Vec<EventValue>,
    pub metadata: EventMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventValue {
    Null,
    Boolean(bool),
    Integer(i64),
    Float(f64),
    String(String),
    Object(ObjectId),
    List(Vec<EventValue>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMetadata {
    pub player: Option<ObjectId>,
    pub location: Option<ObjectId>,
    pub permissions: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum EventPattern {
    /// Match events by exact name
    Exact(String),
    /// Match events by prefix
    Prefix(String),
    /// Match all events from a specific object
    FromObject(ObjectId),
    /// Match all events
    All,
    /// Custom predicate (for complex patterns)
    Custom(String), // Would be compiled to a predicate
}

/// Event subscription handle
pub struct Subscription {
    pub id: Uuid,
    pub pattern: EventPattern,
    pub receiver: UnboundedReceiver<Arc<EchoEvent>>,
}

/// Event store with pub/sub capabilities
pub struct EventStore {
    /// Persistent storage for event history
    events: sled::Tree,
    /// Active subscriptions
    subscriptions: Arc<DashMap<Uuid, (EventPattern, UnboundedSender<Arc<EchoEvent>>)>>,
    /// Event sequence counter
    sequence: sled::Tree,
}

impl EventStore {
    pub fn new(db: &sled::Db) -> Result<Self> {
        let events = db.open_tree("events")?;
        let sequence = db.open_tree("event_sequence")?;

        Ok(Self {
            events,
            subscriptions: Arc::new(DashMap::new()),
            sequence,
        })
    }

    /// Emit an event to all matching subscribers
    pub async fn emit(&self, event: EchoEvent) -> Result<()> {
        // Store event persistently
        let key = self.next_sequence_key()?;
        let value = bincode::serialize(&event)?;
        self.events.insert(key, value)?;

        // Notify subscribers
        let event_arc = Arc::new(event.clone());
        let mut dead_subs = Vec::new();

        for entry in self.subscriptions.iter() {
            let sub_id = *entry.key();
            let (pattern, sender) = entry.value();

            if self.matches_pattern(&event, pattern) {
                if sender.send(event_arc.clone()).is_err() {
                    // Subscriber disconnected
                    dead_subs.push(sub_id);
                }
            }
        }

        // Clean up dead subscriptions
        for id in dead_subs {
            self.subscriptions.remove(&id);
        }

        Ok(())
    }

    /// Subscribe to events matching a pattern
    pub fn subscribe(&self, pattern: EventPattern) -> Subscription {
        let (sender, receiver) = mpsc::unbounded_channel();
        let id = Uuid::new_v4();

        self.subscriptions.insert(id, (pattern.clone(), sender));

        Subscription {
            id,
            pattern,
            receiver,
        }
    }

    /// Unsubscribe from events
    pub fn unsubscribe(&self, sub_id: Uuid) {
        self.subscriptions.remove(&sub_id);
    }

    /// Query historical events
    pub fn query_history(
        &self,
        filter: impl Fn(&EchoEvent) -> bool,
        limit: usize,
    ) -> Result<Vec<EchoEvent>> {
        let mut events = Vec::new();

        for item in self.events.iter().rev().take(limit) {
            let (_, value) = item?;
            let event: EchoEvent = bincode::deserialize(&value)?;
            if filter(&event) {
                events.push(event);
            }
        }

        Ok(events)
    }

    /// Get events in a time range
    pub fn events_between(&self, start_time: u64, end_time: u64) -> Result<Vec<EchoEvent>> {
        self.query_history(
            |e| e.timestamp >= start_time && e.timestamp <= end_time,
            usize::MAX,
        )
    }

    fn matches_pattern(&self, event: &EchoEvent, pattern: &EventPattern) -> bool {
        match pattern {
            EventPattern::Exact(name) => event.name == *name,
            EventPattern::Prefix(prefix) => event.name.starts_with(prefix),
            EventPattern::FromObject(id) => event.source == *id,
            EventPattern::All => true,
            EventPattern::Custom(_predicate) => {
                // TODO: Implement predicate evaluation
                true
            }
        }
    }

    fn next_sequence_key(&self) -> Result<Vec<u8>> {
        let key = b"sequence";
        let seq = self
            .sequence
            .fetch_and_update(key, |old| {
                let num = old
                    .and_then(|bytes| bytes.try_into().ok())
                    .map(u64::from_be_bytes)
                    .unwrap_or(0);
                Some((num + 1).to_be_bytes().to_vec())
            })?
            .unwrap_or_else(|| 0u64.to_be_bytes().to_vec().into());

        Ok(seq.to_vec())
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;

    #[tokio::test]
    async fn test_event_pub_sub() {
        let temp_dir = TempDir::new().unwrap();
        let db = sled::open(temp_dir.path()).unwrap();
        let event_store = EventStore::new(&db).unwrap();

        // Subscribe to all events
        let mut sub = event_store.subscribe(EventPattern::All);

        // Emit an event
        let event = EchoEvent {
            id: EventId::new(),
            name: "PlayerMoved".to_string(),
            source: ObjectId::new(),
            timestamp: 12345,
            args: vec![],
            metadata: EventMetadata {
                player: None,
                location: None,
                permissions: vec![],
            },
        };

        event_store.emit(event.clone()).await.unwrap();

        // Receive the event
        let received = sub.receiver.try_recv().unwrap();
        assert_eq!(received.name, "PlayerMoved");
    }
}
