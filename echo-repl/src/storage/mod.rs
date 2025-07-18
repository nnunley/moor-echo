use anyhow::Result;
use std::path::Path;

pub mod object_store;
pub mod event_store;
pub mod index;

pub use object_store::{ObjectStore, ObjectId, EchoObject, PropertyValue};
pub use event_store::{EventStore, EchoEvent, EventId, EventPattern, Subscription};
pub use index::IndexManager;

/// Complete storage system combining objects, events, and indices
pub struct Storage {
    pub objects: ObjectStore,
    pub events: EventStore,
    pub indices: IndexManager,
}

impl Storage {
    pub fn new(path: impl AsRef<Path>) -> Result<Self> {
        let db = sled::open(&path)?;
        let objects = ObjectStore::new_with_db(db.clone())?;
        let events = EventStore::new(&db)?;
        let indices = IndexManager::new(&db)?;
        
        Ok(Self {
            objects,
            events,
            indices,
        })
    }
}

/// Initialize the storage subsystem
pub async fn init_storage(path: impl AsRef<Path>) -> Result<Storage> {
    Storage::new(path)
}